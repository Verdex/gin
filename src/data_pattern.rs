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

enum Blarg<'a> {
    Emit(&'a Tree),
    Pop,
}

struct Paths<'a> {
    q : Vec<Blarg<'a>>,
    result : Vec<&'a Tree>,
}

impl<'a> Paths<'a> {
    fn new(input : &'a Tree) -> Self {
        Paths{ result : vec![], q : vec![Blarg::Emit(input)] }
    }
}

impl<'a> Iterator for Paths<'a> {
    type Item = Vec<&'a Tree>;
    fn next(&mut self) -> Option<Self::Item> {
        while self.q.len() != 0 {
            let t = self.q.pop().unwrap();
            match t {
                Blarg::Emit(x @ Tree::Leaf(_)) => {
                    let mut ret = self.result.clone();
                    ret.push(x);
                    return Some(ret);
                }, 
                Blarg::Emit(x @ Tree::Node(y, z)) => {
                    self.result.push(x);
                    self.q.push(Blarg::Pop);
                    self.q.push(Blarg::Emit(z)); 
                    self.q.push(Blarg::Emit(y));
                },
                Blarg::Pop => {
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