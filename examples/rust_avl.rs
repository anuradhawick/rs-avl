use rs_avl::AVLTree;

fn main() {
    let mut tree: AVLTree<_> = [30, 10, 20, 40, 50].into_iter().collect();

    println!("ascending: {:?}", tree.iter().collect::<Vec<_>>());
    println!(
        "range 15..=40: {:?}",
        tree.range(15..=40).collect::<Vec<_>>()
    );

    tree.remove(&30);
    println!("after removing 30: {:?}", tree.iter().collect::<Vec<_>>());
}
