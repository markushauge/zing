use ringbuf::Consumer;

pub struct StreamInfo {
    pub sample_rate: f32,
}

pub trait Node {
    fn read(&mut self, buffer: &mut [f32], info: &StreamInfo);
}

pub struct InputNode {
    consumer: Consumer<f32>,
}

impl InputNode {
    pub fn new(consumer: Consumer<f32>) -> Self {
        Self { consumer }
    }
}

impl Node for InputNode {
    fn read(&mut self, buffer: &mut [f32], _: &StreamInfo) {
        if self.consumer.pop_slice(buffer) < buffer.len() {
            eprintln!("Input stream fell behind");
        }
    }
}

pub struct Graph<I, N>
where
    I: Node,
    N: Node,
{
    input: I,
    nodes: Vec<N>,
}

impl<I, N> Graph<I, N>
where
    I: Node,
    N: Node,
{
    pub fn new(input: I, nodes: Vec<N>) -> Self {
        Self { input, nodes }
    }

    pub fn add_node(&mut self, node: N) {
        self.nodes.push(node);
    }

    pub fn remove_node(&mut self, id: usize) {
        self.nodes.remove(id);
    }
}

impl<I, N> Node for Graph<I, N>
where
    I: Node,
    N: Node,
{
    fn read(&mut self, buffer: &mut [f32], info: &StreamInfo) {
        self.input.read(buffer, info);

        for node in &mut self.nodes {
            node.read(buffer, info);
        }
    }
}
