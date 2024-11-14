# Jasmine

## Data Type

### Scalar

| name      | format                               |
| --------- | ------------------------------------ |
| bool      | true,false                           |
| int       | 00000                                |
| float     | 00000.00000, inf, -inf               |
| date      | YYYY-MM-DD                           |
| time      | HH:mm:ss.sss                         |
| datetime  | YYYY-MM-DD[T]HH:mm:ss.sss            |
| timestamp | YYYY-MM-DD[D]HH:mm:ss.sss            |
| duration  | 00000[D]HH:mm:ss.sss,1ns,1s,1m,1h,1d |
| string    | "string"                             |
| cat       | `cat                                 |
| null      | none                                 |

### Mixed List

```
l(1, none, `cat)
```

### Series

| name     | data type   |
| -------- | ----------- |
| bool     | Boolean     |
| f32      | Float32     |
| f64      | Float64     |
| i8       | Int8        |
| i16      | Int16       |
| i32      | Int32       |
| i64      | Int64       |
| u8       | UInt8       |
| u16      | UInt16      |
| u32      | UInt32      |
| u64      | UInt64      |
| date     | Date        |
| datetime | Datetime    |
| duration | Duration    |
| time     | Time        |
| string   | String      |
| cat      | Categorical |
| list     | List        |
| unknown  | Unknown     |

```
// empty series, s(name)
s("",`i8)

// non-empty series
s("name", `bool, true, none, false)
```

### Dataframe

a collection of series

```
// empty series
d(s("series1", `i32), s("series2", `f32))


// non-empty series
d(s("series1", `i32), s("series2", `f32, none, 2, 3.0))
```

### Matrix

a 2d float array

```
// empty matrix
m((), (), ())

// non-empty matrix
m((1, 2), (2, 3), (4, None))
```

### Dictionary

```
// empty dictionary
{}

// non-empty dictionary
{a:1, b:2, c:3}
d(s(`cat, `a, `b, `c), l(1, none, true))
```

### Control Flow

```
if(condition) {
  statement1;
  statement2;
}

while(condition) {
  statement1;
  statement2;
}
```

### Function

```
fn(param1, param2){
  statement1;
  statement2;
}
```

## Expression

### SQL

```
from table [filter condition] [by series1, series2, ...] select series1, series2, ...;
from table [filter condition] [by series1, series2, ...] update series1, series2, ...;
from table filter condition delete all;
from table [filter condition] delete series1, series2, ...;

select(dataframe, (), (), ())
update(dataframe, (), (), ())
delete(dataframe, (), (), ())
```

### Assignment

```
var1: expression1
```

### Unary Operation

```
var1 var2
var1(var2)
```

### Binary Operation

```
var1 !var2 var3
var2(var1, var3)
```
