import clsx from "clsx";
import useDocusaurusContext from "@docusaurus/useDocusaurusContext";
import Layout from "@theme/Layout";
import HomepageFeatures from "@site/src/components/HomepageFeatures";
import Heading from "@theme/Heading";
import useBaseUrl from "@docusaurus/useBaseUrl";
import ThemedImage from "@theme/ThemedImage";
import styles from "./index.module.css";

function HomepageHeader() {
  const { siteConfig } = useDocusaurusContext();
  return (
    <header className={clsx("hero hero--primary", styles.heroBanner)}>
      <div className="container">
        <Heading as="h1" className="hero__title">
          {siteConfig.title}
        </Heading>
        <ThemedImage
          alt="Gerust logo"
          sources={{
            light: useBaseUrl("/img/logo.svg"),
            dark: useBaseUrl("/img/logo-dark-mode.svg"),
          }}
        />
        <p className={styles.heroText}>
          Gerust is a generator for Rust backend projects. It takes care of the
          accidental complexity so you can stay focused on what matters.
        </p>
        <p className={styles.heroText}>
          Gerust projects build on top of{" "}
          <a href="https://github.com/tokio-rs/axum" target="_blank">
            axum
          </a>{" "}
          and{" "}
          <a href="https://github.com/launchbadge/sqlx" target="_blank">
            SQLx
          </a>{" "}
          – proven crates that are widely used in the Rust ecosystem.
        </p>
      </div>
    </header>
  );
}

export default function Home(): JSX.Element {
  return (
    <Layout
      title="Gerust: Rust backend project generator"
      description="Gerust is a project generator for Rust backend projects. It takes care of the accidental complexity so you can stay focused on what matters."
    >
      <HomepageHeader />
      <main>
        <div className="container video-container">
          <iframe
            width="560"
            height="315"
            src="https://www.youtube-nocookie.com/embed/eG8S8QNGP40?si=5OLrNqiVjnyb4zIN"
            title="YouTube video player"
            frameborder="0"
            allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture; web-share"
            referrerpolicy="strict-origin-when-cross-origin"
            allowfullscreen
          ></iframe>
        </div>
        <HomepageFeatures />
      </main>
    </Layout>
  );
}
