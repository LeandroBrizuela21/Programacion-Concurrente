use std::time::Duration;

use async_std::task;
use std::collections::{HashSet, HashMap};
use futures::join;
use futures::future::join_all;

// Este codigo representa un caso de uso real de la asincronia: El Patron Agregador de Servicios,
//  donde un servicio consulta una base de datos y luego pide datos complementarios a varios 
//  microservicios externos para armar una vista completa.

// `join!`: Se utiliza para un numero fijo de tareas.
//    - Los tipos de retorno son heterogeneos (pueden ser distintos).
//    - Devuelve una tupla.  
// `join_all`: Se utiliza para un numero dinamico de tareas.  
//    - Los tipos de retorno son homogeneos (todos iguales).
//    - Devuelve un vector.

// Representa una venta.
struct Sale {
    id: i32,
    user_id: i32, // Clave foranea.
    product_id: i32, // // Clave foranea.
}

struct Product {
    id: i32,
    name: String
}

struct User {
    id: i32,
    email: String
}

// Objeto final que se quiere construir, contiene informacion legible el usuario final.
#[derive(Debug)]
struct SaleView {
    id: i32,
    product: String,
    user: String
}

async fn query_db() -> Vec<Sale> {
    task::sleep(Duration::from_secs(1)).await;
    vec!(
        Sale { id: 1, user_id: 1, product_id: 1 },
        Sale { id: 2, user_id: 1, product_id: 2 },
        Sale { id: 3, user_id: 2, product_id: 3 },
    )
}

async fn find_users_by_ids(ids:HashSet<i32>) -> Vec<User> { 
    // Simula una peticion a un microservicio de usuarios.
    // Tarda 2 segundos, pero como es una API "bulk" (masiva) trae todos los usuarios de una sola vez.
    // Devolvemos aquellos usuarios que coincidan en `id`.
    task::sleep(Duration::from_secs(2)).await;
    vec!(
        User { id: 1, email: String::from("user1@user.com")},
        User { id: 2, email: String::from("user2@user.com")},
        User { id: 3, email: String::from("ignored")}
    ).into_iter().filter(|u| ids.contains(&u.id)).collect()
}

// Simulamos buscar un solo producto.
async fn find_product_by_id(id:i32) -> Product {
    task::sleep(Duration::from_millis(300)).await;
    Product { id, name: String::from("Product ") + id.to_string().as_str()}
}

async fn find_products_by_ids(ids:HashSet<i32>) -> Vec<Product> {
    // Con `join_all` podemos recibir un iterador de Futures. Como no sabemos cuántos productos van
    // a ser solicitados, creamos un monton de peticiones individuales y las disparamos todas a la vez.
    let products_futures = ids.into_iter()
        // Creamos las maquinas de estado sin ejecutarlas.
        .map(|id| find_product_by_id(id)); 
        // Dispara las peticiones al mismo tiempo, solapando las peticiones con un tiempo de 300ms.
        join_all(products_futures).await
}

async fn async_main() -> Vec<SaleView> {

    let sales = query_db().await;
    
    //Extraemos los IDs de los usuarios y productos.
    let user_ids = sales.iter().map(|u| u.user_id).collect();
    let product_ids = sales.iter().map(|u| u.product_id).collect();

    // Preparamos las Maquinas de Estado, aun no se esta ejecutando nada. 
    let users_future = find_users_by_ids(user_ids);
    let products_future = find_products_by_ids(product_ids);

    // - El motor hace poll a ambas tareas, ambas lanzan peticiones de red y devuelven `Pending`.
    // - Los tiempos muertos se solapan simultaneamente sin bloquear el hilo.
    // - A los 300ms termina products; el `join!` lo guarda y sigue esperando a users.
    // - A los 2s termina users; el join! se desarma y desempaqueta la tupla final.
    let (users, products) = join!(users_future, products_future);

    // Convertimos los vectores en HashMap para poder buscar por ID 
    let users_by_id: HashMap<i32, &User> = users.iter().map(|u| (u.id, u)).collect();
    let products_by_id: HashMap<i32, &Product> = products.iter().map(|p| (p.id, p)).collect();

    // Mapeamos las ventas originales cruzando los datos obtenidos.
    sales.iter().map(|s| {
        let p = products_by_id.get(&s.product_id).unwrap();
        let u = users_by_id.get(&s.user_id).unwrap();

        SaleView {
            id: s.id,
            product: p.name.clone(),
            user: u.email.clone()
        }}).collect()
}

fn main() {
    println!("{:?}", task::block_on(async_main()))
}
