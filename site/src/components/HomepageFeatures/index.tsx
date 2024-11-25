import clsx from "clsx";
import Heading from "@theme/Heading";
import CodeBlock from "@theme/CodeBlock";
import styles from "./styles.module.css";

type FeatureItem = {
  title: string;
  Svg?: React.ComponentType<React.ComponentProps<"svg">>;
  code?: { language: string; block: string };
  description: JSX.Element;
};

const FeatureList: FeatureItem[][] = [
  [
    {
      title: "Separation of Concerns",
      code: {
        block: `.
├── Cargo.toml
├── cli
│   └── …
├── config
│   └── …
├── db
│   └── …
├── macros
│   └── …
└── web
    └── …



`,
        language: "",
      },
      description: (
        <>
          Using Cargo workspaces, Gerust separates concerns clearly, improving
          maintainability and compile times.
        </>
      ),
    },
    {
      title: "Clear Folder Structure",
      code: {
        language: "",
        block: `web
├── src
│   ├── controllers
│   │   ├── mod.rs
│   │   └── tasks.rs
│   ├── …
│   ├── middlewares
│   │   ├── auth.rs
│   │   └── mod.rs
│   ├── routes.rs
│   ├── state.rs
│   └── …
└── tests
    └── api
        └── tasks_test.rs`,
      },
      description: (
        <>
          A clear folder structure with defined places for different elements
          supports effective collaboration.
        </>
      ),
    },
    {
      title: "Complete Data Layer",
      code: {
        language: "rust",
        block: `#[derive(Serialize, Debug, Deserialize)]
pub struct Task {
    pub id: Uuid,
    pub description: String,
}

#[derive(Deserialize, Validate, Clone)]
pub struct TaskChangeset {
    #[cfg_attr(feature = "test-helpers", dummy(faker = "Sentence(3..8)"))]
    #[validate(length(min = 1))]
    pub description: String,
}

pub async fn load(id: Uuid, executor: impl sqlx::Executor<'_, Database = Postgres>) -> Result<Task, crate::Error> {
  …`,
      },
      description: (
        <>
          Gerust comes with a complete data layer based on SQLx with entities,
          migrations, validations, changesets, and more.
        </>
      ),
    },
  ],
  [
    {
      title: "Testing",
      code: {
        language: "rust",
        block: `#[db_test]
async fn test_read_all(context: &DbTestContext) {
    let task_changeset: TaskChangeset = Faker.fake();
    create_task(task_changeset.clone(), &context.db_pool).await.unwrap();

    let response = context
        .app
        .request("/tasks")
        .method(Method::GET)
        .send()
        .await;

    let tasks: TasksList = response.into_body().into_json::<TasksList>().await;
    assert_that!(tasks, len(eq(1)));
}`,
      },
      description: (
        <>
          Gerust projects are fully testable with abstractions built-in for
          database-backed tests with complete isolation.
        </>
      ),
    },
    {
      title: "Migrations & Seed Data",
      code: {
        language: "",
        block: `» cargo db migrate -e test
ℹ️  Migrating test database…
     Applied migration 1732531458.
✅ 1 migrations applied.

» cargo db seed        
ℹ️  Seeding development database…
✅ Seeded database successfully.    





    
    `,
      },
      description: (
        <>Gerust generates and runs migrations and maintains seed data.</>
      ),
    },
    {
      title: "Scaffolding",
      code: {
        language: "",
        block: `» cargo generate help
A CLI tool to generate project files.
      
Usage: generate [OPTIONS] <COMMAND>

Commands:
  middleware            Generate a middleware
  controller            Generate a controller
  controller-test       Generate a test for a controller
  entity                Generate an entity
  crud-controller       Generate an example CRUD controller
  crud-controller-test  Generate a test for a CRUD controller
  …


`,
      },
      description: (
        <>
          Gerust comes with tooling for generating e.g. controllers,
          middlewares, and entities – with scaffolding for maximum productivity.
        </>
      ),
    },
  ],
];

function Feature({ title, Svg, code, description }: FeatureItem) {
  return (
    <div className={clsx("col col--4", styles.feature)}>
      <div className="text--center">
        {Svg ? <Svg className={styles.featureSvg} role="img" /> : <></>}
        {code ? (
          <CodeBlock language={code.language}>{code.block}</CodeBlock>
        ) : (
          <></>
        )}
      </div>
      <div className="text--center padding-horiz--md">
        <Heading as="h3">{title}</Heading>
        <p>{description}</p>
      </div>
    </div>
  );
}

export default function HomepageFeatures(): JSX.Element {
  return (
    <section className={styles.features}>
      <div className="container">
        {FeatureList.map((row) => (
          <div className="row">
            {row.map((props, idx) => (
              <Feature key={idx} {...props} />
            ))}
          </div>
        ))}
      </div>
    </section>
  );
}
