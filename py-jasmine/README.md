# jasminum

an implementation of the data analytics programming language jasmine powered by [Polars](https://pola.rs/)

## Installation

```
pip install jasminum
```

## Start a jasminum Process

jasminum

## Data Type

### Scalar

| name      | format                               |
| --------- | ------------------------------------ |
| BOOLEAN   | true,false                           |
| INT       | 00000                                |
| FLOAT     | 00000.00000, inf, -inf               |
| DATE      | YYYY-MM-DD                           |
| TIME      | HH:mm:ss.sss                         |
| DATETIME  | YYYY-MM-DD[T]HH:mm:ss.sss            |
| TIMESTAMP | YYYY-MM-DD[D]HH:mm:ss.sssssssss      |
| DURATION  | 00000[D]HH:mm:ss.sss,1ns,1s,1m,1h,1d |
| STRING    | "string"                             |
| CAT       | `` `cat` ``                          |
| NONE      | none                                 |

`datetime` and `timestamp` are with timezone information. To convert a timezone `nyi`

- `` t ~as `Asia/Hong_Kong` ``
- `` as(t, `Asia/Hong_Kong`) ``

### List(Mixed Data Types)

```
l[1, none, `cat`]
```

### Series

| name      | data type      |
| --------- | -------------- |
| bool      | Boolean        |
| f32       | Float32        |
| f64       | Float64        |
| i8        | Int8           |
| i16       | Int16          |
| i32       | Int32          |
| i64       | Int64          |
| u8        | UInt8          |
| u16       | UInt16         |
| u32       | UInt32         |
| u64       | UInt64         |
| date      | Date           |
| datetime  | Datetime("ms") |
| timestamp | Datetime("ns") |
| duration  | Duration       |
| time      | Time           |
| string    | String         |
| cat       | Categorical    |
| list      | List           |
| unknown   | Unknown        |

```
// empty series
`i8`$[]

// non-empty series
[true, none, false]
[0i8, 1, 2]
```

### Dataframe

a collection of series

```
// empty series
df[series1= `i32`$[], series2= `f32`$[]]


// non-empty series
df[series1 = `i32`$[ , , ], series2 = `f32`$[none, 2.0, 3.0]]
df[series1 = [0i32, , ], series2 = [none, 2.0, 3.0]]
```

### Matrix

a 2d float array

```
// empty matrix
x[[], [], []]

// non-empty matrix
x[[1, 2], [2, 3], [4, None]]
```

### Dictionary

```
// empty map
{}

// non-empty map
{a:1, b:2, c:3}
d[[`a`, `b`, `c`], [1, none, true]]
d[]
```

### Variable Name

- Starts with alphabets, the var name can include alphabets, number and "\_"
- Starts with CJK character, the var name can include CJK character, alphabets, number and "\_"

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
fn(param1, param2, ...){
  statement1;
  statement2;
}
```

### Function Call

```
fn(arg1, arg2, ...)
```

## Expression

### SQL

```
from table
  [ filter {condition[,condition]} ]
  [ group {series1, series2, ...} ]
  [
    select {series1, series2, ...}
    | update {series1, series2, ...}
    | delete {series1, series2, ...}
  ]
  [ sort {series1, -series2, ...} ]
  [ take number ]

select(dataframe, (), (), ())
update(dataframe, (), (), ())
delete(dataframe, (), (), ())
```

### Assignment

```
var1 = expression1
```

### Unary Operation

```
var1 var2
var1(var2)
```

### Binary Operation

```
var1 ~var2 var3
var2(var1, var3)
```

### Iteration

#### Each

```
var1 ~each series1
var1 ~each list1
var1 ~each dataframe1

// apply each for 1st param
f2(var1) ~each var2
// apply each for 2nd param
f2(,var2) ~each var1
```
