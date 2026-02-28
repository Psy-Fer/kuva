Output docs/src/assets/terminal/volcano.gif
Set Width 1200
Set Height 620
Set FontSize 14
Set Theme "Dracula"
Set TypingSpeed 50ms
Set PlaybackSpeed 1.5

Sleep 500ms
Type "./target/release/visus volcano examples/data/volcano.tsv --name-col gene --x-col log2fc --y-col pvalue --title Volcano --x-label log2FC --y-label neg_log10p --terminal --term-width 108 --term-height 32"
Sleep 3s
Enter
Sleep 10s
