use vertex::*;

pub type BoardData = Vec<Vec<Sign>>;

#[derive(Debug, Clone)]
pub struct PseudoBoard {
    pub width: usize,
    pub height: usize,
    pub data: BoardData
}

impl PseudoBoard {
    pub fn new(data: BoardData) -> PseudoBoard {
        let height = data.len();
        let width = match data.len() {
            0 => 0,
            _ => data[0].len()
        };

        PseudoBoard {width, height, data}
    }

    pub fn get(&self, Vertex(x, y): Vertex) -> Option<Sign> {
        match self.data.get(y) {
            None => None,
            Some(row) => row.get(x).cloned()
        }
    }

    pub fn set(&mut self, Vertex(x, y): Vertex, sign: Sign) {
        self.data[y][x] = sign;
    }

    pub fn is_point_chain(&self, vertex: Vertex) -> bool {
        let sign = self.get(vertex);

        !get_neighbors(vertex)
        .into_iter()
        .any(|v| self.get(v) == sign)
    }

    pub fn get_connected_component(&self, vertex: Vertex, signs: &[Sign]) -> Vec<Vertex> {
        fn inner(
            board: &PseudoBoard,
            vertex: Vertex,
            signs: &[Sign],
            mut result: Vec<Vertex>
        ) -> Vec<Vertex> {
            let neighbors = get_neighbors(vertex);
            let sign = board.get(vertex).unwrap();

            for neighbor in neighbors.into_iter() {
                let s = match board.get(neighbor) {
                    Some(x) => x,
                    None => continue
                };

                if !signs.contains(&s) || result.contains(&neighbor) {
                    continue;
                }

                result.push(neighbor);
                result = inner(board, neighbor, signs, result);
            }

            result
        }

        inner(&self, vertex, signs, vec![vertex])
    }

    pub fn get_related_chains(&self, vertex: Vertex) -> Vec<Vertex> {
        let sign = self.get(vertex).unwrap();
        let area = self.get_connected_component(vertex, &vec![sign, 0]);

        area.into_iter()
        .filter(|&v| self.get(v).unwrap() == sign)
        .collect()
    }

    pub fn get_chain(&self, vertex: Vertex) -> Vec<Vertex> {
        let sign = self.get(vertex).unwrap();

        self.get_connected_component(vertex, &vec![sign])
    }

    pub fn has_liberties(&self, vertex: Vertex) -> bool {
        fn inner(
            board: &PseudoBoard,
            vertex: Vertex,
            mut visited: Vec<Vertex>,
            sign: Sign
        ) -> (Vec<Vertex>, bool) {
            let neighbors = get_neighbors(vertex);
            let mut friendly_neighbors = vec![];

            for neighbor in neighbors.into_iter() {
                let s = match board.get(neighbor) {
                    Some(s) => s,
                    None => continue
                };
                
                if s == 0 {
                    return (visited, true);
                } else if s == sign {
                    friendly_neighbors.push(neighbor);
                }
            }

            visited.push(vertex);

            for neighbor in friendly_neighbors.into_iter() {
                if visited.contains(&neighbor) {
                    continue;
                }

                visited = match inner(board, neighbor, visited, sign) {
                    (visited, true) => return (visited, true),
                    (visited, false) => visited
                };
            }

            (visited, false)
        }

        inner(&self, vertex, vec![], self.get(vertex).unwrap()).1
    }

    pub fn make_pseudo_move(&mut self, sign: Sign, vertex: Vertex) -> Option<Vec<Vertex>> {
        let neighbors = get_neighbors(vertex);
        let mut check_capture = false;
        let mut check_multiple_dead_chains = false;

        if neighbors.iter().all(|&neighbor| {
            match self.get(neighbor) {
                None => true,
                Some(s) if s == sign => true,
                _ => false
            }
        }) {
            return None;
        }

        self.set(vertex, sign);

        if !self.has_liberties(vertex) {
            if self.is_point_chain(vertex) {
                check_multiple_dead_chains = true;
            } else {
                check_capture = true;
            }
        }

        let mut dead = vec![];
        let mut dead_chains = 0;

        for &neighbor in neighbors.iter() {
            match self.get(neighbor) {
                Some(s) if s != -sign => continue,
                _ if self.has_liberties(neighbor) => continue,
                _ => ()
            }

            let chain = self.get_chain(neighbor);
            dead_chains += 1;

            for c in chain.into_iter() {
                self.set(c, 0);
                dead.push(c);
            }
        }

        if check_multiple_dead_chains && dead_chains <= 1 {
            for d in dead.iter() {
                self.set(*d, -sign);
            }

            self.set(vertex, 0);
            return None;
        }

        if check_capture && dead.len() == 0 {
            self.set(vertex, 0);
            return None;
        }

        Some(dead)
    }

    pub fn get_floating_stones(&self) -> Vec<Vertex> {
        let mut done = vec![];
        let mut result = vec![];

        let vertices = (0..self.width).flat_map(|x| {
            (0..self.height).map(move |y| Vertex(x, y))
        });

        for vertex in vertices {
            if self.get(vertex).unwrap() != 0 || done.contains(&vertex) {
                continue;
            }

            let pos_area = self.get_connected_component(vertex, &vec![0, -1]);
            let neg_area = self.get_connected_component(vertex, &vec![0, 1]);
            let pos_dead = pos_area.iter().cloned()
                .filter(|&v| self.get(v).unwrap() == -1).collect::<Vec<_>>();
            let neg_dead = neg_area.iter().cloned()
                .filter(|&v| self.get(v).unwrap() == 1).collect::<Vec<_>>();
            let pos_diff = pos_area.iter().cloned()
                .filter(|v| !pos_dead.contains(v) && !neg_area.contains(v)).count();
            let neg_diff = neg_area.iter().cloned()
                .filter(|v| !neg_dead.contains(v) && !pos_area.contains(v)).count();

            let mut sign = 0;

            if neg_diff <= 1 && neg_dead.len() <= pos_dead.len() {
                sign -= 1;
            }

            if pos_diff <= 1 && pos_dead.len() <= neg_dead.len() {
                sign += 1;
            }

            let (actual_area, mut actual_dead) = match sign {
                1 => (pos_area, pos_dead),
                -1 => (neg_area, neg_dead),
                _ => (self.get_chain(vertex), vec![])
            };

            for &v in actual_area.iter() {
                done.push(v);
            }

            result.append(&mut actual_dead);
        }

        result
    }
}