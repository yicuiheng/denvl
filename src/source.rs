use std::path::Path;

// Source の要件
// - ファイルパスから生成される
//   - ファイルは UTF8 形式とみなす
//   - ファイル内容は denvl 言語であるという前提をおかない。つまりデータの持ち方に構文の情報を使わないことにする
// - 行番号と列番号と修正の長さと修正後の文字列が与えられて、修正後のソースコードを表現できる
// - 修正されたがまだ構文解析していない領域を取得できる
// - 指定した位置から一文字ずつ取得できる
// - 必要最小限のデータを持つ
// - 行番号と列番号からエラー箇所のコードを表示したいが、これはエラーの位置情報でソートしてワンパスでエラーメッセージを構成できたら充分
// Source の内部実装メモ
// - Source は行の列とみなせる

pub struct Source {
    pub buffer: Vec<char>,
}

impl Source {
    pub fn new(path: &Path) -> Result<Self, std::io::Error> {
        let src = std::fs::read_to_string(path)?;
        let buffer = src
            .lines()
            .flat_map(|line| {
                let mut line: Vec<_> = line.chars().collect();
                line.push('\n');
                line
            })
            .collect();
        Ok(Source { buffer })
    }

    // 事前条件: check_pos_validity(pos)
    pub fn at(&self, pos: Position) -> char {
        assert!(self.check_pos_validity(pos));
        self.buffer[pos.0]
    }

    pub fn check_pos_validity(&self, pos: Position) -> bool {
        pos.0 < self.buffer.len()
    }

    pub fn range(&self) -> Range {
        Range {
            start: Position(0),
            end: Position(self.buffer.len()),
        }
    }

    pub fn get(&self, range: &Range) -> &[char] {
        &self.buffer[range.start.0..range.end.0]
    }
}

// 0 origin
#[derive(Clone, Debug, PartialEq, Eq, Copy, PartialOrd, Ord)]
pub struct Position(pub usize);

impl Position {
    pub fn start() -> Self {
        Position(0)
    }

    pub fn advance(&mut self, n: usize) {
        self.0 += n;
    }

    pub fn backward(&mut self, n: usize) {
        self.0 -= n;
    }

    pub fn distance(lhs: Position, rhs: Position) -> usize {
        lhs.0 - rhs.0
    }
}

impl std::ops::Add<usize> for Position {
    type Output = Position;
    fn add(self, other: usize) -> Position {
        Position(self.0 + other)
    }
}

// 常に start <= end が成り立つ
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Range {
    pub start: Position,
    pub end: Position,
}

impl Range {
    pub fn is_empty(&self) -> bool {
        assert!(self.start <= self.end);
        self.width() == 0
    }

    pub fn width(&self) -> usize {
        assert!(self.start <= self.end);
        Position::distance(self.end, self.start)
    }

    // pred が成り立つか self.is_empty() が true になるまで start を進める
    pub fn skip_until<F>(&mut self, pred: F)
    where
        F: Fn(&Range) -> bool,
    {
        while !self.is_empty() && !pred(self) {
            self.start.advance(1)
        }
    }
}

pub fn starts_with(source: &Source, str: &str, range: &Range) -> bool {
    let mut range = *range;
    for expected in str.chars() {
        if range.is_empty() {
            return false;
        }
        let actual = source.at(range.start);
        if expected != actual {
            return false;
        }
        range.start.advance(1);
    }
    true
}

pub fn match_(source: &Source, str: &str, range: &Range) -> bool {
    starts_with(source, str, range) && str.len() == range.width()
}

#[cfg(test)]
impl Source {
    pub fn from_str(str: &str) -> Self {
        Self {
            buffer: str
                .lines()
                .flat_map(|line| {
                    let mut line: Vec<_> = line.chars().collect();
                    line.push('\n');
                    line
                })
                .collect(),
        }
    }
}
