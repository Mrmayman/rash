Essentially performs this computation

```python
pi = 0.0;
d = 1.0;
for i in range(10000000):
    pi += (8.0 * (float(i) % 2) - 4.0) / d
    d += 2.0
```

- `pi.sb3`: Same as above
- `pi functions.sb3`: Loop contents moved into a separate function in hot path
- `pi functions arguments.sb3`: Same as above, but 8.0 and 2.0 are function arguments not constants
