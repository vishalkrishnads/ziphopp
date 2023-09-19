import Header from './components/Header/Header'
import styles from './page.module.css'

export default function Home() {

  return (
    <main className={styles.main}>
      <div className={styles.app}>
        <div className={styles.left}>
          <div className={styles.contents}>
            <Header />
            <div className={styles.filepane}></div>
            <div className={styles.recentpane}></div>
          </div>
        </div>
        <div className={styles.right}></div>
      </div>
    </main>
  )
}
