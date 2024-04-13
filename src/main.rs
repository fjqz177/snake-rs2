use device_query::{DeviceQuery, DeviceState, Keycode};
use rand::Rng;
use std::process::Command;
use std::time::{Duration, Instant};
use tokio::{spawn, time};

const WIDTH: usize = 100;
const HEIGHT: usize = 25;

type Map = [[&'static str; WIDTH]; HEIGHT];

// 方向
#[derive(Clone, Copy, PartialEq)]
enum Direction {
    UP,
    DOWN,
    LEFT,
    RIGHT,
}

/// 蛇
struct Snake {
    // ● 蛇头
    head: [usize; 2],
    // 速度
    speed: u64,
    // 蛇身
    body: Vec<[usize; 2]>,
}

/// 食物
struct Food {
    position: [usize; 2],
    eat: bool,
}
/// 将食物加入地图
fn add_food_to_map(food: &Food, map: &mut Map) {
    if food.eat {
        // 食物如果被吃了，不要加到地图
        return;
    }
    let position = food.position;
    map[position[0]][position[1]] = "■";
}

/// 随机生成食物
/// 随机算法这里仅是简单实现，生成食物的时候需要避开蛇的身体
fn random_new_food(food: &mut Food, _snake: &Snake, map: &mut Map) {
    if !food.eat {
        // 如果当前的食物没有被吃掉，则不生成新的食物
        return;
    }
    // let head = snake.head;
    // let body = &(snake.body);
    let mut rng = rand::thread_rng();
    let x = rng.gen_range(1..HEIGHT - 1);
    let y = rng.gen_range(1..WIDTH - 1);
    map[x][y] = "▣";
    food.position = [x, y];
    food.eat = false;
}

/// 创建食物
fn create_food() -> Food {
    let position = [5, 8];
    return Food {
        position,
        eat: false,
    };
}

/// 创建蛇
fn create_snake() -> Snake {
    let head = [5, 6];
    let speed = 200;
    let body = vec![[5, 5], [5, 4]];
    return Snake { head, speed, body };
}

/// 将蛇加入地图
fn add_snake_to_map(snake: &Snake, map: &mut Map) {
    let head = snake.head;
    map[head[0]][head[1]] = "●";
    for i in snake.body.iter() {
        map[i[0]][i[1]] = "■";
    }
}

/// 移动蛇
fn move_snake(snake: &mut Snake, food: &mut Food, map: &mut Map, direction: Direction) {
    let before_head = snake.head;
    // 是否吃到了食物
    let eat: bool;
    match direction {
        Direction::UP => {
            // 蛇头往前移动一格，以前的蛇头变成蛇身的位置
            snake.head[0] -= 1;
            eat = snake.head == food.position;
            map[before_head[0]][before_head[1]] = "■";
            map[snake.head[0]][snake.head[1]] = "●";
        }
        Direction::DOWN => {
            // 蛇头往前移动一格，以前的蛇头变成蛇身的位置
            snake.head[0] += 1;
            eat = snake.head == food.position;
            map[before_head[0]][before_head[1]] = "■";
            map[snake.head[0]][snake.head[1]] = "●";
        }
        Direction::LEFT => {
            // 蛇头往前移动一格，以前的蛇头变成蛇身的位置
            snake.head[1] -= 1;
            eat = snake.head == food.position;
            map[before_head[0]][before_head[1]] = "■";
            map[snake.head[0]][snake.head[1]] = "●";
        }
        Direction::RIGHT => {
            // 先判断是否吃到了食物
            snake.head[1] += 1;
            eat = snake.head == food.position;
            // 蛇头往前移动一格，以前的蛇头变成蛇身的位置
            map[before_head[0]][before_head[1]] = "■";
            map[snake.head[0]][snake.head[1]] = "●";
        }
    }
    // 蛇身去掉最后一个元素
    if !snake.body.is_empty() && !eat {
        let snake_foot = snake.body.remove(snake.body.len() - 1);
        map[snake_foot[0]][snake_foot[1]] = " ";
    }
    if eat {
        food.eat = true;
    }
    // 插入蛇身第一个元素
    snake.body.insert(0, before_head);
}

/// 游戏是否结束
fn is_game_over(snake: &Snake) -> bool {
    let head = snake.head;
    // 如果蛇头触及到边界 触发游戏结束
    if head[0] == 0 || head[0] == HEIGHT - 1 || head[1] == 0 || head[1] == WIDTH - 1 {
        return true;
    }
    let body_list = &snake.body;
    if body_list.is_empty() {
        return false;
    }
    // 蛇头触及到蛇的身体
    for body in body_list.iter() {
        if body[0] == head[0] && body[1] == head[1] {
            return true;
        }
    }
    return false;
}

/// 打印地图
fn print_map(map: Map) {
    for i in 0..map.len() {
        for j in 0..map[i].len() {
            print!("{}", map[i][j]);
        }
        println!();
    }
}

/// 创建地图
fn create_map() -> Map {
    let block = "■";
    let empty = " ";
    let mut map = [[empty; WIDTH]; HEIGHT];
    for i in 0..map.len() {
        for j in 0..map[i].len() {
            // 第一行和最后一行边界
            if i == 0 || i == map.len() - 1 {
                map[i][j] = block;
            }

            // 第一列和最后一列边界
            if j == 0 || j == map[i].len() - 1 {
                map[i][j] = block;
            }
        }
    }
    return map;
}

/// 清空屏幕
fn clear_screen() {
    Command::new("cmd.exe")
        .arg("/c")
        .arg("cls")
        .status()
        .expect("clear error!");
}

async fn input_handler(tx: tokio::sync::mpsc::Sender<Keycode>) {
    let device_state = DeviceState::new();
    loop {
        let keycodes = device_state.get_keys();
        for key in keycodes {
            tx.send(key).await.unwrap();
        }
        time::sleep(Duration::from_millis(10)).await;
    }
}

async fn game_loop(
    mut map: Map,
    mut snake: Snake,
    mut food: Food,
    mut rx: tokio::sync::mpsc::Receiver<Keycode>, // 改为使用Receiver
) {
    let mut current_direction = Direction::RIGHT;
    let mut last_frame = Instant::now();

    loop {
        let now = Instant::now();
        let elapsed = now - last_frame;
        if elapsed < Duration::from_millis(snake.speed as u64) {
            tokio::time::sleep(Duration::from_millis(snake.speed as u64) - elapsed).await;
        }

        // 处理接收到的按键，更新方向
        while let Ok(key) = rx.try_recv() {
            match key {
                Keycode::W => {
                    if current_direction != Direction::DOWN {
                        current_direction = Direction::UP
                    }
                }
                Keycode::A => {
                    if current_direction != Direction::RIGHT {
                        current_direction = Direction::LEFT
                    }
                }
                Keycode::S => {
                    if current_direction != Direction::UP {
                        current_direction = Direction::DOWN
                    }
                }
                Keycode::D => {
                    if current_direction != Direction::LEFT {
                        current_direction = Direction::RIGHT
                    }
                }
                _ => (),
            }
        }

        move_snake(&mut snake, &mut food, &mut map, current_direction); // 移动蛇

        last_frame = Instant::now();
        clear_screen();
        add_snake_to_map(&snake, &mut map);
        add_food_to_map(&food, &mut map);
        random_new_food(&mut food, &snake, &mut map);
        print_map(map);

        if is_game_over(&snake) {
            println!("Game Over!");
            break;
        }
    }
}

#[tokio::main]
async fn main() {
    let map = create_map();
    let snake = create_snake();
    let food = create_food();
    let (tx, rx) = tokio::sync::mpsc::channel(100);

    // 创建输入处理协程
    spawn(input_handler(tx));

    // 创建游戏循环协程，现在直接使用rx，而不是input_events队列
    game_loop(map, snake, food, rx).await;
}
