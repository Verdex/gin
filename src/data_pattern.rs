/*
    It's a depth first search except you keep track of each item you see 
    then return those items at the bottom

    However, you also need to keep some of the items and drop others
    each time you backtrack and take a new path


*/

#[derive(Debug)]
enum Tree {
    Node(Box<Tree>, Box<Tree>) ,
    Leaf(u8),
}


fn visit<'a>(input : &'a Tree) -> Vec<&'a Tree> {
    fn h<'a>(input : &'a Tree, a : &mut Vec<&'a Tree>) {
        match input { 
            x @ Tree::Leaf(_) => a.push(x),
            x @ Tree::Node(y, z) => {
                a.push(x);
                h(y, a);
                h(z, a);
            },
        } 

    }
    let mut ret = vec![];
    h(input, &mut ret);
    ret
}

fn iter_visit<'a>(input : &'a Tree) -> Vec<&'a Tree> {
    let mut q = vec![];
    let mut ret = vec![];
    q.push(input);

    while q.len() != 0 {
        let t = q.pop().unwrap();
        match t {
            x @ Tree::Leaf(_) => ret.push(x),
            x @ Tree::Node(y, z) => {
                ret.push(x);
                q.push(z); // recursive version calls immediately, but iterative waits until next time through the loop
                           // at which point we'll have already pushed the other item.  So we want to reverse the order 
                           // of the push to get the same iteration order.
                q.push(y);
            },
        }
    }
    ret
}

enum PathAction<'a, T> {
    Emit(&'a T),
    Pop,
}

struct Paths<'a, T> {
    q : Vec<PathAction<'a, T>>,
    result : Vec<&'a T>,
}

pub enum Echo<'a, T> {
    Terminal(&'a T),
    Node(&'a T, Vec<&'a T>),
} 

/*trait Pathite<'a> {
    fn choose(&self) -> Echo<'a, Self>;
}

impl<'a, T> Paths<'a, T> {
    fn new(input : &'a T) -> Self {
        Paths{ result : vec![], q : vec![PathAction::Emit(input)] }
    }
}

impl<'a, T> Iterator for Paths<'a, T> {
    type Item = Vec<&'a T>;
    fn next(&mut self) -> Option<Self::Item> {
        while self.q.len() != 0 {
            let t = self.q.pop().unwrap();
            match t {
                PathAction::Emit(x) => {
                    match x.choose() {
                        Echo::Terminal(y) => {
                            let mut ret = self.result.clone();
                            ret.push(x);
                            return Some(ret);
                        },
                        Echo::Node(y, ys) => {
                            self.result.push(y);
                            self.q.push(PathAction::Pop);
                            for ylet in ys {
                                self.q.push(PathAction::Emit(ylet)); 
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
}*/

/*impl<'a> Paths<'a, Tree> {
    fn new(input : &'a Tree) -> Self {
        Paths{ result : vec![], q : vec![PathAction::Emit(input)] }
    }
}

impl<'a> Iterator for Paths<'a, Tree> {
    type Item = Vec<&'a Tree>;
    fn next(&mut self) -> Option<Self::Item> {
        while self.q.len() != 0 {
            let t = self.q.pop().unwrap();
            match t {
                PathAction::Emit(x @ Tree::Leaf(_)) => {
                    let mut ret = self.result.clone();
                    ret.push(x);
                    return Some(ret);
                }, 
                PathAction::Emit(x @ Tree::Node(y, z)) => {
                    self.result.push(x);
                    self.q.push(PathAction::Pop);
                    self.q.push(PathAction::Emit(z)); 
                    self.q.push(PathAction::Emit(y));
                },
                PathAction::Pop => {
                    self.result.pop();
                },
            }
        }
        None
    }
}*/

/*macro_rules! blarg {
    () => {

    };

    (node, $($nexts:ident)*, $q:expr, $result:expr, $x:pat, $($w:tt)*) => {
        PathAction::Emit(out @ $x) => {
            $result.push(out);
            $q.push(PathAction::Pop);
            $(
                $q.push(PathAction::Emit($nexts)); 
            )*
        },
        blarg!($($w)*)
    };

    (terminal, $result:expr, $x:pat, $($w:tt)*) => {
        PathAction::Emit(out @ $x) => {
            let mut ret = $result.clone();
            ret.push(out);
            return Some(ret);
        }, 
        blarg!($($w)*)
    };
}

macro_rules! path {
    ($target:ty, $($w:tt)*) => {
        impl<'a> Iterator for Paths<'a, $target> {
            type Item = Vec<&'a $target>;
            fn next(&mut self) -> Option<Self::Item> {
                while self.q.len() != 0 {
                    let t = self.q.pop().unwrap();
                    match t {
                        blarg!($($w)*)
                        PathAction::Pop => {
                            self.result.pop();
                        },
                    }
                }
                None
            }
        }
    };
}

path!(Tree, terminal, )*/

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

        let paths = Paths::new(&input).into_iter().collect::<Vec<_>>();

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

    #[test]
    fn blarg() {
        let x = Tree::Node(
            Box::new(Tree::Leaf(1)),
            Box::new(
                Tree::Node(
                    Box::new(Tree::Leaf(2)),
                    Box::new(Tree::Leaf(3))
                )
            )
        );

       /* let y = visit(&x);
        let w = iter_visit(&x);

        for ylet in y {
            println!("{:?}", ylet);
        }

        println!("\n\n");

        for wlet in w {
            println!("{:?}", wlet);
        }*/

        let ps = Paths::new(&x);

        println!("====");

        for p in ps {
            for plet in p {
                println!("{:?}", plet);
            }
            println!("\n\n\n");
        }
    }
}