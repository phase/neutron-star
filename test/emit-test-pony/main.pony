actor Main
  new create(env: Env) =>
    let a = ActorWithIntField
    a.test(X(3, 4))

class X
  var _a: I32
  var _b: I32
  new create(a: I32, b: I32) =>
    _a = a
    _b = b
  fun getA(): I32 => _a
  fun getB(): I32 => _b

actor ActorWithIntField
  var _test: I32
  new create() =>
    _test = 0
  be test(x: X iso) =>
    _test = x.getA()
