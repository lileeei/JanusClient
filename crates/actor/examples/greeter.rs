fn main() {
    // 创建 Actor 系统
    let system = ActorSystem::new("janus", ActorSystemConfig::default());

    // 创建顶层 Actor
    let greeter = system.create_actor("greeter", || GreeterActor::new("Hello"));

    // 创建子 Actor
    let mut result = block_on(async {
        // 发送消息并等待响应
        greeter
            .send(Greet {
                name: "World".to_string(),
            })
            .await
    });

    // 打印结果
    match result {
        Ok(message) => println!("Received: {}", message),
        Err(e) => eprintln!("Error: {:?}", e),
    }

    // 关闭系统
    block_on(system.shutdown());
}

// 示例 Actor
struct GreeterActor {
    prefix: String,
    count: usize,
}

impl GreeterActor {
    fn new(prefix: &str) -> Self {
        Self {
            prefix: prefix.to_string(),
            count: 0,
        }
    }
}

impl Actor for GreeterActor {
    type Context = BasicContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        println!("GreeterActor started!");

        // 创建子 Actor
        let child = ctx.spawn("child", || ChildActor);

        // 调度一个任务
        ctx.schedule(Duration::from_secs(1), |actor, _| {
            println!("Scheduled task executed, count: {}", actor.count);
            actor.count += 1;
        });
    }
}

// 消息定义
struct Greet {
    name: String,
}

impl Message for Greet {
    type Result = String;
}

// 消息处理
impl Handler<Greet> for GreeterActor {
    type Result = String;

    fn handle(&mut self, msg: Greet, _ctx: &mut Self::Context) -> Self::Result {
        self.count += 1;
        format!("{}, {}! (count: {})", self.prefix, msg.name, self.count)
    }
}

// 子 Actor
struct ChildActor;

impl Actor for ChildActor {
    type Context = BasicContext<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        println!("ChildActor started!");
    }
}
