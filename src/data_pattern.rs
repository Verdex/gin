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
    x : fn(&'a T) -> Echo<'a, T>,
    result : Vec<&'a T>,
}

pub enum Echo<'a, T> {
    Terminal(&'a T),
    Node(&'a T, Vec<&'a T>),
} 


impl<'a, T> Paths<'a, T> {
    fn new(input : &'a T, x : fn(&'a T) -> Echo<'a, T>) -> Self {
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
                        Echo::Terminal(w) => {
                            let mut ret = self.result.clone();
                            ret.push(w);
                            return Some(ret);
                        },
                        Echo::Node(w, ws) => {
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

/*impl<'a> Iterator for Paths<'a, Tree> {
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

    fn blarg<'a>(t : &'a Tree) -> Echo<'a, Tree> {
        match t {
            x @ Tree::Leaf(_) => Echo::Terminal(x),
            x @ Tree::Node(y, z) => Echo::Node(x, vec![z, y]),
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

    #[test]
    fn test_blarg() {
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

        let ps = Paths::new(&x, blarg);

        println!("====");

        for p in ps {
            for plet in p {
                println!("{:?}", plet);
            }
            println!("\n\n\n");
        }
    }
}