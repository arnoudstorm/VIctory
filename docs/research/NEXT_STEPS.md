# Next steps

0. **Past the prison intro (E19):** the post-intro load dies with the title's TileHeap full (3,977/3,977 MB). Find the texture-streaming feedback the title reads back from the GPU and check it is right; a wrong read-back asking for the finest mips everywhere would fill the pool (and lower-detail textures would follow from fixing it).

1. **Background recompiles off** needs Windows Developer Mode (`SetBackgroundProcessingMode` returned an error without it); with it on, rerun `bg-off` and check `ok=true`.
1b. **Isolate the combined run's gain** (748 ms per frame in `all-new`):
   one run with only `VICTORY_D3D12_BACKGROUND=0` added to the baseline,
   against the ten-run baseline (mean 1,073 ms, range 759-1,605).
2. **The remaining waits** (5 per frame, 170-250 ms): 2-2.5 are "another
   copy" (a refresh of one window while a flight writes an overlapping
   one): the duplicate windows again (E14, E17). Fewer duplicates means
   fewer waits; a safe merge must go through every holder of a window.
3. **Ambient level**: 0.6 measured (+47-53 % in dark regions); next a warmer tint for dusk and the shade strength (`VICTORY_AMBIENT_SHADE`).
4. **Skipping the intro** with held A (`hold_a_keys.txt` in the session
   notes), with other programs closed, to see whether the Prestream load
   fits in memory now.
5. With permission: build DexNom side by side and run it once normally,
   once with `--rt` settings, as Series X, with its own data folder.
6. Belobug `03629f3f` (quad reads, AMD helper lanes) after a GPU pixel
   test that shows the defect.
