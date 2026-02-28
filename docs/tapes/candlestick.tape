Output docs/src/assets/terminal/candlestick.gif
Set Width 1200
Set Height 620
Set FontSize 14
Set Theme "Dracula"
Set TypingSpeed 50ms
Set PlaybackSpeed 1.5

Sleep 500ms
Type "./target/release/visus candlestick examples/data/candlestick.tsv --label-col date --open-col open --high-col high --low-col low --close-col close --title StockPrice --x-label Date --y-label Price --terminal --term-width 108 --term-height 32"
Sleep 3s
Enter
Sleep 10s
