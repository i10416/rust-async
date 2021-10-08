packages/block_on はまだ1つの future しかはしらせられない.
executor を実装して複数のfutureを非同期に実行できるようにする.

https://www.codercto.com/a/101177.html


https://tech-blog.optim.co.jp/entry/2020/03/05/160000#Pin%E3%81%AF%E9%9D%9E%E5%90%8C%E6%9C%9F%E3%83%97%E3%83%AD%E3%82%B0%E3%83%A9%E3%83%9F%E3%83%B3%E3%82%B0%E3%82%88%E3%82%8A
Rust標準では、Unpinを実装しない、つまり「ムーブしたら絶対アカン😡」オブジェクトは非同期関数の戻り値と非同期ブロックのみです。

Pin
通常はコンパイラの最適化によって参照は動かされうるが、それを防ぎたい. 
future::pollは内部ではunsafeでメモリをderefしているのでコンパイラの最適化によって参照先が動かされると困る.
futureがもつfutureの内部の状態が変わっても参照は同一のものを指定したい.

ハードウェアからの通知をfutureがある特定のメモリへの書き込みと考えるとわかりやすいかもしれない.
