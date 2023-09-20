import Image from 'next/image'
import logo from '../../assets/logo.png'
import styles from './header.module.css'

export default function Header() {
    return <div className={styles.header}>
        <div className={styles.margin} />
        <div className={styles.logo}>
            <Image
                src={logo}
                alt={""}
                style={{ width: '9vw', height: '9vw'  }}
                unoptimized
             />
        </div>
        <div className={styles.name}>
            <h1><u>Zip</u>Hopp</h1>
            <code><h3>v0.1</h3></code>
        </div> 
    </div>
}