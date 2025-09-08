use std::cmp::min;
use core_log::init_tracer;
use crate::utils::hasher::{Hasher, HasherSha256};

pub struct MerkleTree<T : Hasher> {
    layers: Vec<Vec<T::Hash>>,
}

impl <T : Hasher> MerkleTree<T> {

    pub fn from_data(data: Vec<Vec<u8>>) -> MerkleTree<T> {
        return Self::from_hashes(
            data.iter()
                .map(|item| T::hash(item.as_slice()))
                .collect()
        )
    }

    pub fn from_hashes(hashes: Vec<T::Hash>) -> MerkleTree<T> {
        let size = hashes.len();
        let mut layers = Vec::<Vec<T::Hash>>::from(vec![hashes]);
        let height = Self::max_height(size);

        for i in 0..height {
            let layer = Self::build_layer(&layers[i]);
            layers.push(layer);
        }

        layers.reverse();

        return MerkleTree::<T> { layers };
    }

    fn max_height(size: usize) -> usize {
        return (size as f32).log2().ceil() as usize
    }

    fn build_layer(data: &Vec<T::Hash>) -> Vec<T::Hash> {
        let mut layers = Vec::<T::Hash>::new();

        for i in 0..data.len() {
            let left_id = i * 2;
            let mut right_id = i * 2 + 1;

            if left_id >= data.len() {
                break
            }

            if right_id >= data.len() {
                right_id = left_id
            }

            let hash = T::concat_and_hash(
                data[left_id].as_ref(),
                data[right_id].as_ref(),
            );

            layers.push(hash);
        }

        return layers
    }
}

impl <T : Hasher> MerkleTree<T> {

    pub fn root(&self) -> T::Hash {
        return self.layers[0][0]
    }

    pub fn gen_proof(&self, index: usize) -> Option<Vec<T::Hash>> {
        if self.layers.is_empty() {
            return None
        }

        let hashes = self.layers.len() - 1;
        let mut result = Vec::<T::Hash>::with_capacity(hashes);
        let mut pointer = index;

        for i in (1..=hashes).rev() {
            let id = if pointer % 2 == 0 { pointer + 1 } else { pointer - 1 };
            let hash = self.clone_layer_proof(i, id)?;
            result.push(hash);
            pointer /= 2;
        }

        return Some(result)
    }

    fn clone_layer_proof(&self, layer: usize, index: usize) -> Option<T::Hash> {
        let layer = self.layers.get(layer)?;
        let result = layer.get(min(layer.len() - 1, index))?;
        return Some(result.clone())
    }

    pub fn proof_root(&self, index: usize, data: &[u8], proof: &Vec<T::Hash>) -> T::Hash {
        let hash = T::hash(data);
        let mut pointer = index;

        let result = proof.iter()
            .fold(hash, |ptr, item| {
                if pointer % 2 == 0 {
                    pointer /= 2;
                    T::concat_and_hash(ptr.as_ref(), item.as_ref())
                } else {
                    pointer /= 2;
                    T::concat_and_hash(item.as_ref(), ptr.as_ref())
                }
            });

        return result
    }
}

#[tokio::test]
async fn test_app_verifier() {
    let _guard = init_tracer();
    let data = vec![
        String::from("1").into_bytes(),
        String::from("2").into_bytes(),
        String::from("3").into_bytes(),
        String::from("4").into_bytes(),
        String::from("5").into_bytes(),
        String::from("6").into_bytes(),
        String::from("7").into_bytes(),
        String::from("8").into_bytes(),
        String::from("9").into_bytes(),
    ];

    let tree = MerkleTree::<HasherSha256>::from_data(data);
    assert_eq!(tree.layers.len(), 5);

    let proof = tree.gen_proof(1).expect("");
    let root = tree.proof_root(1, "2".as_bytes(), &proof);
    assert_eq!(&root, &tree.root());

    let proof = tree.gen_proof(5).expect("");
    let root = tree.proof_root(5, "6".as_bytes(), &proof);
    assert_eq!(&root, &tree.root());

    let proof = tree.gen_proof(15).expect("");
    let root = tree.proof_root(15, "9".as_bytes(), &proof);
    assert_eq!(&root, &tree.root());
}

#[tokio::test]
async fn test_generated() {
    let _guard = init_tracer();
    let data: Vec<[u8; 32]> = vec![
        hex::decode("a0f9abf1e9364aa569fa42d6b1c0e737d0207f8859eac865a042e842e6e971fe").expect("").try_into().expect(""),
        hex::decode("d8cb30504fb40a73fc7e20ca82c40bdc0649ca90dbeb91d9b1edf10d76078038").expect("").try_into().expect(""),
        hex::decode("3198b209c0209f0f1b2e7aa8397de4ea6ae7b335f6429b3f4d7fd09cf0267b84").expect("").try_into().expect(""),
        hex::decode("fa4ccb748dede1cd4d5fe68a2873710884693a8bb3a46bca3b9e395c26c88f12").expect("").try_into().expect(""),
        hex::decode("f50562ac98c06a3d68fed1cd874638609576ecbab805d5d276b71b41f0624276").expect("").try_into().expect(""),
        hex::decode("5fc7d6b1633e04476d98f72e3817a3be7e6d509fac0267a36bb16ac667eb2ab9").expect("").try_into().expect(""),
        hex::decode("f2b22a7ef6e03b6ad63b3c8a1f0e8091d03b5926a7dd0c2b041d717f4e3d16e1").expect("").try_into().expect(""),
        hex::decode("44656b5e2de96ebd6f33e045df32ee8d04109a415c475e5cee6d76cdf815ef50").expect("").try_into().expect(""),
    ];

    let tree = MerkleTree::<HasherSha256>::from_hashes(data);
    assert_eq!(tree.layers.len(), 4);
    assert_eq!(
        hex::decode("9030a5969c65bcb28495d55baebf60be5f174f5b44ead7adf2b1b647ae0153dd").expect(""),
        tree.layers.get(2).expect("").get(0).expect("")
    );
}
