[]
[9]
[10]
[11]
[12]
[13]
[32]
[194, 133]
[194, 160]
[225, 154, 128]
[226, 128, 128]
[226, 128, 129]
[226, 128, 130]
[226, 128, 131]
[226, 128, 132]
[226, 128, 133]
[226, 128, 134]
[226, 128, 135]
[226, 128, 136]
[226, 128, 137]
[226, 128, 138]
[226, 128, 168]
[226, 128, 169]
[226, 128, 175]
[226, 129, 159]
[227, 128, 128]
match s.next() {
None => panic!("found []"),
//opportunity for sharing
9 => match s.next() {
None => panic!("found [9]"),
//opportunity for sharing
_ => panic!("default"),
}
10 => match s.next() {
None => panic!("found [10]"),
//opportunity for sharing
_ => panic!("default"),
}
11 => match s.next() {
None => panic!("found [11]"),
//opportunity for sharing
_ => panic!("default"),
}
12 => match s.next() {
None => panic!("found [12]"),
//opportunity for sharing
_ => panic!("default"),
}
13 => match s.next() {
None => panic!("found [13]"),
//opportunity for sharing
_ => panic!("default"),
}
32 => match s.next() {
None => panic!("found [32]"),
_ => panic!("default"),
}
194 => match s.next() {
133 => match s.next() {
None => panic!("found [194, 133]"),
//opportunity for sharing
_ => panic!("default"),
}
160 => match s.next() {
None => panic!("found [194, 160]"),
_ => panic!("default"),
}
_ => panic!("default"),
}
225 => match s.next() {
154 => match s.next() {
128 => match s.next() {
None => panic!("found [225, 154, 128]"),
_ => panic!("default"),
}
_ => panic!("default"),
}
_ => panic!("default"),
}
226 => match s.next() {
128 => match s.next() {
128 => match s.next() {
None => panic!("found [226, 128, 128]"),
//opportunity for sharing
_ => panic!("default"),
}
129 => match s.next() {
None => panic!("found [226, 128, 129]"),
//opportunity for sharing
_ => panic!("default"),
}
130 => match s.next() {
None => panic!("found [226, 128, 130]"),
//opportunity for sharing
_ => panic!("default"),
}
131 => match s.next() {
None => panic!("found [226, 128, 131]"),
//opportunity for sharing
_ => panic!("default"),
}
132 => match s.next() {
None => panic!("found [226, 128, 132]"),
//opportunity for sharing
_ => panic!("default"),
}
133 => match s.next() {
None => panic!("found [226, 128, 133]"),
//opportunity for sharing
_ => panic!("default"),
}
134 => match s.next() {
None => panic!("found [226, 128, 134]"),
//opportunity for sharing
_ => panic!("default"),
}
135 => match s.next() {
None => panic!("found [226, 128, 135]"),
//opportunity for sharing
_ => panic!("default"),
}
136 => match s.next() {
None => panic!("found [226, 128, 136]"),
//opportunity for sharing
_ => panic!("default"),
}
137 => match s.next() {
None => panic!("found [226, 128, 137]"),
//opportunity for sharing
_ => panic!("default"),
}
138 => match s.next() {
None => panic!("found [226, 128, 138]"),
//opportunity for sharing
_ => panic!("default"),
}
168 => match s.next() {
None => panic!("found [226, 128, 168]"),
//opportunity for sharing
_ => panic!("default"),
}
169 => match s.next() {
None => panic!("found [226, 128, 169]"),
//opportunity for sharing
_ => panic!("default"),
}
175 => match s.next() {
None => panic!("found [226, 128, 175]"),
_ => panic!("default"),
}
_ => panic!("default"),
}
129 => match s.next() {
159 => match s.next() {
None => panic!("found [226, 129, 159]"),
_ => panic!("default"),
}
_ => panic!("default"),
}
_ => panic!("default"),
}
227 => match s.next() {
128 => match s.next() {
128 => match s.next() {
None => panic!("found [227, 128, 128]"),
_ => panic!("default"),
}
_ => panic!("default"),
}
_ => panic!("default"),
}
_ => panic!("default"),
}
