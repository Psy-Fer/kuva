Output docs/src/assets/terminal/manhattan.gif
Set Width 1200
Set Height 620
Set FontSize 14
Set Theme "Dracula"
Set TypingSpeed 50ms
Set PlaybackSpeed 1.5

Sleep 500ms
Type "./target/release/kuva manhattan examples/data/gene_stats.tsv --chr-col chr --pvalue-col pvalue --title Manhattan --x-label Chromosome --y-label neg_log10p --terminal --term-width 108 --term-height 32"
Sleep 3s
Enter
Sleep 10s
