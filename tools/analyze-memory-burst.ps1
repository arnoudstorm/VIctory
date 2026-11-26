[CmdletBinding()]
param([string]$Events,[string]$Regions,[long]$StartUtcMs,[long]$EndUtcMs,[switch]$SelfTest)
$ErrorActionPreference='Stop'

function Analyze-Burst($EventRows,$RegionRows,[long]$Start,[long]$End) {
 if($End -le $Start){throw 'End must follow start'}
 $scans=@($RegionRows | Where-Object {$null -ne $_.totals} | Sort-Object utc_ms)
 $before=$scans|Where-Object {$_.utc_ms -le $Start}|Select-Object -Last 1
 $after=$scans|Where-Object {$_.utc_ms -le $End}|Select-Object -Last 1
 if(!$before -or !$after){throw 'Need complete scans at or before both interval boundaries'}
 $complete=@{};foreach($scan in $scans){$complete[[string]$scan.utc_ms]=$true}
 $window=@($EventRows|Where-Object {$null -ne $_.utc_us -and $_.utc_us -ge $before.utc_ms*1000 -and $_.utc_us -le $after.utc_ms*1000})
 $changes=@($RegionRows|Where-Object {$null -ne $_.allocation_base -and $complete.ContainsKey([string]$_.utc_ms) -and $_.utc_ms -gt $before.utc_ms -and $_.utc_ms -le $after.utc_ms})
 $growth=foreach($group in ($changes|Group-Object allocation_base)) {
  $delta=@(0L,0L,0L,0L,0L)
  foreach($row in $group.Group){for($i=0;$i -lt 5;$i++){$delta[$i]+=[long]$row.delta[$i]}}
  if($delta[1] -le 0){continue}
  $last=$group.Group|Sort-Object utc_ms|Select-Object -Last 1
  $base=[long]$last.allocation_base;$span=[long](($last.counts|Measure-Object -Sum).Sum)
  # An observed address in a growing allocation is correlation, never proof
  # that that request created all commitment or owns its backing pages.
  $related=@($window|Where-Object {
   ($_.address -ge $base -and $_.address -lt $base+$span) -or
   ($_.result -ge $base -and $_.result -lt $base+$span)
  })
  [pscustomobject]@{AllocationBase=$base;PrivateCommitDelta=$delta[1];ReservedDelta=$delta[0];MappedCommitDelta=$delta[2];ImageCommitDelta=$delta[3];ObservedAddressMatches=$related.Count;RelatedApis=@($related.api|Sort-Object -Unique);Attribution='Unattributed; address matches alone do not establish ownership'}
 }
 $apis=foreach($group in ($window|Where-Object {$null -ne $_.requested_bytes}|Group-Object api)) {
  [pscustomobject]@{Api=$group.Name;Calls=$group.Count;RequestedBytesNotNetCommit=[long](($group.Group.requested_bytes|Measure-Object -Sum).Sum)}
 }
 $totalDelta=for($i=0;$i -lt 5;$i++){[long]$after.totals[$i]-[long]$before.totals[$i]}
 $drops=($EventRows.dropped_total|Measure-Object -Maximum).Maximum
 $stackGroups=@($window|Where-Object {$null -ne $_.stack -and ($_.stack|Measure-Object -Maximum).Maximum -gt 0}|Group-Object {($_.stack|Where-Object {$_ -ne 0}) -join ','}|Sort-Object Count -Descending|Select-Object -First 16|ForEach-Object {
  [pscustomobject]@{Calls=$_.Count;RawFrames=$_.Group[0].stack;RequestedBytesNotNetCommit=[long](($_.Group.requested_bytes|Measure-Object -Sum).Sum)}
 })
 [pscustomobject]@{
  RequestedStartUtcMs=$Start;RequestedEndUtcMs=$End;EffectiveStartUtcMs=$before.utc_ms;EffectiveEndUtcMs=$after.utc_ms
  ByteColumnOrder=@('reserved','private committed','mapped committed','image committed','other committed')
  RegionTotalDeltas=$totalDelta;TopPrivateGrowth=@($growth|Sort-Object PrivateCommitDelta -Descending|Select-Object -First 32)
  ObservedApis=@($apis);TopObservedStacks=$stackGroups;MaximumReportedDroppedEvents=$drops
  ModuleMetadata=@($RegionRows|Where-Object {$null -ne $_.module_base -and $_.utc_ms -le $after.utc_ms}|Group-Object module_base|ForEach-Object {$_.Group|Sort-Object utc_ms|Select-Object -Last 1})
  Limitations=@('Snapshots are not atomic; effective boundaries differ from requested boundaries.','Only complete region scans are used; missed sub-snapshot allocation lifetimes remain unknown.','Virtual-memory requests are not net commitment; overlapping commits must not be summed.','MEM_MAPPED aliases and image/mapped copy-on-write prevent equality with process PrivateUsage.','All growth remains unattributed until source/lifetime/stack evidence establishes ownership.','A zero dropped counter does not prove the asynchronously terminated event tail is complete.')
 }
}

if($SelfTest){
 $m=1048576L
 $syntheticRegions=@(
  [pscustomobject]@{utc_ms=1000;allocation_base=65536;counts=@((32*$m),0,0,0,0);delta=@((32*$m),0,0,0,0)},
  [pscustomobject]@{utc_ms=1000;totals=@((32*$m),0,0,0,0)},
  [pscustomobject]@{utc_ms=2000;allocation_base=65536;counts=@((16*$m),(16*$m),0,0,0);delta=@((-16*$m),(16*$m),0,0,0)},
  [pscustomobject]@{utc_ms=2000;allocation_base=4294967296L;counts=@(0,(8*$m),0,0,0);delta=@(0,(8*$m),0,0,0)},
  [pscustomobject]@{utc_ms=2000;totals=@((16*$m),(24*$m),0,0,0)},
  # This partial scan must be ignored: it has no totals record.
  [pscustomobject]@{utc_ms=2500;allocation_base=8589934592L;counts=@(0,(64*$m),0,0,0);delta=@(0,(64*$m),0,0,0)}
 )
 $syntheticEvents=@(
  [pscustomobject]@{api='VirtualAlloc';utc_us=1500000;address=65536;requested_bytes=16*$m;result=65536;dropped_total=0;stack=@(1234,0)},
  [pscustomobject]@{api='VirtualAlloc';utc_us=1600000;address=65536;requested_bytes=16*$m;result=65536;dropped_total=0;stack=@(1234,0)}
 )
 $result=Analyze-Burst $syntheticEvents $syntheticRegions 1100 2700
 if($result.RegionTotalDeltas[1] -ne 24*$m){throw 'Double-counted requests or included incomplete scan'}
 if($result.ObservedApis[0].RequestedBytesNotNetCommit -ne 32*$m){throw 'Lost the independent requested-byte total'}
 $unobserved=@($result.TopPrivateGrowth|Where-Object AllocationBase -eq 4294967296L)
 if($unobserved.Count -ne 1 -or $unobserved[0].ObservedAddressMatches -ne 0 -or $unobserved[0].Attribution -notlike 'Unattributed*'){throw 'Invented attribution for unobserved allocation'}
 if($result.EffectiveEndUtcMs -ne 2000){throw 'Used an incomplete scan'}
 $result|ConvertTo-Json -Depth 8
 return
}
if(!$Events -or !$Regions){throw 'Specify Events and Regions JSONL files'}
$bad=0
function Read-JsonLines([string]$path){
 foreach($line in [IO.File]::ReadLines((Resolve-Path -LiteralPath $path).Path)){
  try {$line|ConvertFrom-Json -ErrorAction Stop}catch{$script:bad++}
 }
}
$eventRows=@(Read-JsonLines $Events);$regionRows=@(Read-JsonLines $Regions)
$report=Analyze-Burst $eventRows $regionRows $StartUtcMs $EndUtcMs
$report|Add-Member NoteProperty MalformedLinesSkipped $bad
$report|ConvertTo-Json -Depth 8
