import Image from 'next/image'
import styles from './recent.module.css'
import recents from '../../assets/recent.png'
import open from '../../assets/open.png'

export default function Recent({name, path, onClick}) {
    return <div onClick={onClick} className={styles.recent}>
      <div className={styles.margin} />
      <div className={styles.icon}>
        <Image
          src={recents}
          alt={''}
          style={{ width: '6vh', height: '6vh'  }}
          unoptimized 
        />
      </div>
      <div className={styles.info}>
        <h4>{name}</h4>
        <p>{path}</p>
      </div>
      <div className={styles.open}>
        <Image
          src={open}
          alt={''}
          style={{ width: '4vh', height: '4vh'  }}
          unoptimized 
        />
      </div>
      <div className={styles.margin} />
    </div>
  }