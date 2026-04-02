# kuva candlestick

OHLC candlestick chart.

**Input:** label, open, high, low, close columns (and optionally volume).

| Flag | Default | Description |
|---|---|---|
| `--label-col <COL>` | `0` | Period label column |
| `--open-col <COL>` | `1` | Open price column |
| `--high-col <COL>` | `2` | High price column |
| `--low-col <COL>` | `3` | Low price column |
| `--close-col <COL>` | `4` | Close price column |
| `--volume-col <COL>` | — | Optional volume column |
| `--volume-panel` | off | Show volume bar panel below price chart |
| `--candle-width <F>` | `0.7` | Body width as a fraction of slot |
| `--color-up <CSS>` | green | Bullish candle color |
| `--color-down <CSS>` | red | Bearish candle color |
| `--color-doji <CSS>` | `#888888` | Doji candle color |

```bash
kuva candlestick candlestick.tsv \
    --label-col date --open-col open --high-col high \
    --low-col low --close-col close

kuva candlestick candlestick.tsv \
    --label-col date --open-col open --high-col high \
    --low-col low --close-col close \
    --volume-col volume --volume-panel
```

---

*See also: [Shared flags](./index.md#shared-flags) — output, appearance, axes, log scale.*
