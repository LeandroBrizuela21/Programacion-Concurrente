use std::thread;
use std::time::{Duration, Instant};

fn main() {

    let data = [7, 3, 2, 16, 24, 4, 11, 9];

    // Rayon crea un "grupo de workers (hilos)" listos para recibir tareas en paralelo. 
    let _ = rayon::ThreadPoolBuilder::new().build_global();

    // Devuelve un punto en el tiempo que lo utilizaremos para medir cuanto tiempo ha pasado.
    let start = Instant::now();
    let merged = mergesort(&data);

    println!("Tiempo total: {:?}", start.elapsed());
    println!("Lista ordenada: {:?}", merged);
}

fn mergesort(data: &[i32]) -> Vec<i32> {

    // Obligamos que el hilo quede dormido por dos segundos.
    // NOTA: Se usa para simular el "trabajo pesado" ya que  dividir el trabajo en multiples hilos 
    // con `rayon::join` es en realidad mas lento que hacerlo de forma tradicional en un solo hilo
    // si la lista a ordenar es pequeña. 
    thread::sleep(Duration::from_secs(2));

    let mid = data.len() / 2;
    
    // Caso base para detener la recursión.
    if mid == 0 {
        return data.to_vec();
    }

    // Nos quedamos con las mitades correspondientes.
    let left_data = &data[..mid];
    let right_data = &data[mid..];

    // En lugar de ejecutar primero el lado izquierdo y luego el lado derecho, `rayon::join`
    // lanza ambas funciones (con tareas independientes) al mismo tiempo en hilos separados 
    // ya instanciados (poolthread) y espera a que ambas terminen.
    let (left, right) = rayon::join(|| mergesort(left_data), || mergesort(right_data));

    // Combina los resultados de las mitades.
    merge(left, right)
}

fn merge(left: Vec<i32>, right: Vec<i32>) -> Vec<i32> {
    let mut left_index = 0;
    let mut right_index = 0;
    let mut ret_index = 0;
    let mut ret = vec![0; left.len() + right.len()];

    while left_index < left.len() && right_index < right.len() {
        if left[left_index] <= right[right_index] {
            ret[ret_index] = left[left_index];
            ret_index += 1;
            left_index += 1;
        } else {
            ret[ret_index] = right[right_index];
            ret_index += 1;
            right_index += 1;
        }
    }

    if left_index < left.len() {
        ret[ret_index..].copy_from_slice(&left[left_index..]);
    }
    if right_index < right.len() {
        ret[ret_index..].copy_from_slice(&right[right_index..]);
    }

    ret
}