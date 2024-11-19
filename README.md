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
l[1, none, `cat]
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
`i8$s[]

// non-empty series
s[true, none, false]
s[0i8, 1, 2]
```

### Dataframe

a collection of series

```
// empty series
d[series1: `i32$s[], series2: `f32$s[]]


// non-empty series
d[series1: `i32$s[], series2: `f32$s[none, 2, 3.0]]
```

### Matrix

a 2d float array

```
// empty matrix
x[[], [], []]

// non-empty matrix
x[[1, 2], [2, 3], [4, None]]
```

### Dictionary(Hashmap)

```
// empty map
{}

// non-empty map
{a:1, b:2, c:3}
m[[`a, `b, `c], [1, none, true]]
m[]
```

### Variable Name

Not allow single char variable name `(ASCII_ALPHANUMERIC | CJK){2,}`

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
from table
  [ filter {condition[,condition]}]
  [ group {series1, series2, ...}]
  [ select {series1, series2, ...}
  | update {series1, series2, ...}
  | delete {series1, series2, ...}]
  [ sort {series1, series2, ...}]
  [ take number]

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

### Iteration

#### Each

```
var1 each series1
var1 each list1
var1 each dataframe1

// apply each for 1st param
f2(var1) each var2
// apply each for 2nd param
f2(,var2) each var1
```
