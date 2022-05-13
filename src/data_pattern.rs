
#[derive(Debug)]
enum Tree {
    Node(Box<Tree>, Box<Tree>) ,
    Leaf(u8),
}

enum PathAction<'a, T> {
    Emit(&'a T),
    Pop,
}

pub enum ConsType<'a, T> {
    Leaf(&'a T),
    Node(&'a T, Vec<&'a T>),
} 

pub struct Paths<'a, T> {
    q : Vec<PathAction<'a, T>>,
    x : fn(&'a T) -> ConsType<'a, T>,
    result : Vec<&'a T>,
}

impl<'a, T> Paths<'a, T> {
    fn new(input : &'a T, x : fn(&'a T) -> ConsType<'a, T>) -> Self {
        Paths{ result : vec![], q : vec![PathAction::Emit(input)], x }
    }
}

impl<'a> Iterator for Paths<'a, Tree> {
    type Item = Vec<&'a Tree>;
    fn next(&mut self) -> Option<Self::Item> {
        while self.q.len() != 0 {
            let t = self.q.pop().unwrap();
            match t {
                PathAction::Emit(x) => {
                    let r = self.x;
                    match r(x) {
                        ConsType::Leaf(w) => {
                            let mut ret = self.result.clone();
                            ret.push(w);
                            return Some(ret);
                        },
                        ConsType::Node(w, ws) => {
                            self.result.push(w);
                            self.q.push(PathAction::Pop);
                            for wlet in ws {
                                self.q.push(PathAction::Emit(wlet)); 
                            }
                        },
                    }
                }, 
                PathAction::Pop => {
                    self.result.pop();
                },
            }
        }
        None
    }
}

#[cfg(test)]
mod test {
    use super::*;

    macro_rules! proj {
        ($input:expr => $pattern:pat => $($output:tt)+) => {
            let $($output)+ = match $input {
                $pattern => $($output)+,
                _ => panic!("projection failure"),
            };
        };
    }

    fn blarg<'a>(t : &'a Tree) -> ConsType<'a, Tree> {
        match t {
            x @ Tree::Leaf(_) => ConsType::Leaf(x),
            x @ Tree::Node(y, z) => ConsType::Node(x, vec![z, y]),
        }
    }

    #[test]
    fn path_should_create_paths() {
        let input = Tree::Node(
            Box::new(Tree::Leaf(1)),
            Box::new(
                Tree::Node(
                    Box::new(Tree::Leaf(2)),
                    Box::new(Tree::Leaf(3))
                )
            )
        );

        let paths = Paths::new(&input, blarg).into_iter().collect::<Vec<_>>();

        assert_eq!( paths.len(), 3 );

        // path 1
        assert_eq!( paths[0].len(), 2 );
        proj!( paths[0][0] => Tree::Node( a00, b00 ) => (a00, b00) );

        assert!( matches!( **a00, Tree::Leaf(1) ) );
        assert!( matches!( **b00, Tree::Node(_, _) ) );

        assert!( matches!( paths[0][1], Tree::Leaf(1) ) );

        // path 2
        assert_eq!( paths[1].len(), 3 );
        proj!( paths[1][0] => Tree::Node( a10, b10 ) => (a10, b10) );

        assert!( matches!( **a10, Tree::Leaf(1) ) );
        assert!( matches!( **b10, Tree::Node(_, _) ) );

        proj!( paths[1][1] => Tree::Node( a11, b11 ) => (a11, b11) );
        assert!( matches!( **a11, Tree::Leaf(2) ) );
        assert!( matches!( **b11, Tree::Leaf(3) ) );

        assert!( matches!( paths[1][2], Tree::Leaf(2) ) );

        // path 3
        assert_eq!( paths[2].len(), 3 );
        proj!( paths[2][0] => Tree::Node( a20, b20 ) => (a20, b20) );

        assert!( matches!( **a20, Tree::Leaf(1) ) );
        assert!( matches!( **b20, Tree::Node(_, _) ) );

        proj!( paths[1][1] => Tree::Node( a21, b21 ) => (a21, b21) );
        assert!( matches!( **a21, Tree::Leaf(2) ) );
        assert!( matches!( **b21, Tree::Leaf(3) ) );

        assert!( matches!( paths[2][2], Tree::Leaf(3) ) );
    }
}