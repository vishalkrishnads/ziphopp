import Image from 'next/image'
import Header from './components/Header/Header'
import Recent from './components/Recent/Recent'
import styles from './page.module.css'
import zip from './assets/zipicon.png'

export default function Home() {

  return (
    <main className={styles.main}>
      <div className={styles.app}>
        <div className={styles.left}>
          <div className={styles.contents}>
            <Header />
            <div className={styles.filepane}>
              {/* <button>Open file</button> */}
              <div className={styles.file}>
                <div className={styles.margin} />
                <div className={styles.icon}>
                  <Image
                    src={zip}
                    alt={''}
                    // style={{ width: '8vw', height: '8vw'  }}
                    unoptimized
                  />
                </div>
                <div className={styles.info}>
                  <h4>File name.zip</h4>
                  <h6>/home/user/Downloads/filename.zip</h6>
                  <p>14KB, uncompresses to 17KB</p>
                  <div><p>Open another</p></div>
                </div>
                <div className={styles.margin} />
              </div>
            </div>
            <div className={styles.recentpane}>
              <div className={styles.margin} />
              <div className={styles.pane}>
                <div className={styles.header}>
                  <h3>You recently opened...</h3>
                </div>
                <div className={styles.recents}>
                  
                </div>
              </div>
              <div className={styles.margin} />
            </div>
          </div>
        </div>
        <div className={styles.right}></div>
      </div>
    </main>
  )
}
