mod linked_list;
// mod linked_list_2;

fn main() {
    env_logger::init();

    let mut ll = linked_list::LinkedList1::<i32>::new();
    ll.insert(123);
    ll.insert(234);

    // let a = ll.head.as_ref().unwrap();
    // a.next.as_ref().unwrap().print();

    ll.traverse();

    log::info!("dd");
}
