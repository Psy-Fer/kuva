Output docs/src/assets/terminal/dot.gif
Set Width 1200
Set Height 620
Set FontSize 14
Set Theme "Dracula"
Set TypingSpeed 50ms
Set PlaybackSpeed 1.5

Sleep 500ms
Type "./target/release/kuva dot examples/data/dot.tsv --x-col pathway --y-col cell_type --size-col pct_expressed --color-col mean_expr --title DotPlot --x-label Pathway --y-label CellType --terminal --term-width 108 --term-height 32"
Sleep 3s
Enter
Sleep 10s
