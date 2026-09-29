import React, {type ReactNode} from 'react';
import Layout from '@theme/Layout';
import Link from '@docusaurus/Link';
import useBaseUrl from '@docusaurus/useBaseUrl';
import styles from './index.module.css';

const paths = [
  ['01', 'Start your first session', 'Install the agent, connect your gateway, sign in, and verify a real response.', '/getting-started/'],
  ['02', 'Deploy the platform', 'Follow the architecture, network boundaries, identity services, and gateway policies.', '/guides/architecture/'],
  ['03', 'Configure identity', 'Set up Keycloak, broker Microsoft Entra ID, or use workspace API keys.', '/configuration/keycloak/'],
  ['04', 'Prove the whole path', 'Check credentials, authorization, inference, security policy, and remote tools.', '/validation/acceptance/'],
];

export default function Home(): ReactNode {
  return (
    <Layout title="Your agent for AI Gateway" description="Prisma AIRS Harness: a local terminal agent for Prisma AIRS AI Gateway, with enterprise identity, governed inference, and gateway-connected tools.">
      <main>
        <section className={styles.hero} aria-labelledby="hero-title">
          <div className={styles.heroCopy}>
            <p className={styles.eyebrow}>PRISMA AIRS / TERMINAL AGENT</p>
            <h1 id="hero-title">Local control.<br /><span>Gateway intelligence.</span></h1>
            <p className={styles.lead}>An agent built for Prisma AIRS AI Gateway. Bring your workspace, sign in with your organization, and put governed models and tools to work.</p>
            <div className={styles.actions}>
              <Link className="button button--primary button--lg" to="/getting-started/">Get started →</Link>
              <Link className={styles.secondary} to="/guides/architecture/">Explore the architecture ↗</Link>
            </div>
            <p className={styles.platforms}>APPLE SILICON · LINUX X64 · LINUX ARM64</p>
          </div>
          <div className={styles.artwork}>
            <img src={useBaseUrl('/img/brand-logo.png')} alt="Prisma AIRS Harness shield and prism spectrum" width="1254" height="1254" fetchPriority="high" />
            <div className={styles.pillRow}><span className={styles.pill}>Organization SSO</span><span className={styles.pill}>Governed models</span><span className={styles.pill}>Gateway tools</span></div>
          </div>
        </section>
        <section className={styles.paths} aria-labelledby="paths-title">
          <div className={styles.sectionIntro}><p className={styles.eyebrow}>FROM FIRST LOGIN TO OPERATIONS</p><h2 id="paths-title">A clear path through the platform.</h2><p>Start with the task in front of you. Each guide includes the context, configuration, and checks you need.</p></div>
          <div className={styles.grid}>{paths.map(([number, title, description, to]) => <Link className={styles.path} to={to} key={number}><span className={styles.number}>{number}</span><h3>{title}</h3><p>{description}</p><span className={styles.arrow} aria-hidden="true">↗</span></Link>)}</div>
        </section>
        <section className={styles.quick}><div><p className={styles.eyebrow}>KEEP IT CLOSE</p><h2>Less searching. More doing.</h2><p>Copy the commands for daily work, or look up the exact flags shipped with the harness.</p></div><div className={styles.actions}><Link className="button button--primary" to="/operations/cheat-sheet/">Open the cheat sheet</Link><Link to="/reference/airs/">Command reference →</Link></div></section>
      </main>
    </Layout>
  );
}
