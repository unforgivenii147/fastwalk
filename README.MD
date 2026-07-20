# fastwalk
i originally named it fastwalk, but it was taken 
so i named it fastwalk

# install
maturin build

# usage
it accept Path or string input 
and returns list of Path
```python
from fastwalk import walk_parallel
for path in walk_parallel("/"):
   print(path.name)

```
