import styles from './password.module.css'

export default function PwdModal({onCancel, onSuccess}) {
    return <div className={styles.modal}>
        <div className={styles.content}>
            <div className={styles.margin} />
            <div className={styles.contents}>
                <div className={styles.header}>
                    <p>This file is password protected</p>
                </div>
                <div className={styles.field}>
                    <input type={'text'} placeholder={'Type your password'} />
                </div>
                <div className={styles.buttons}>
                    <h4 onClick={onCancel}>Cancel</h4>
                    <button>Open</button>
                </div>
            </div>
            <div className={styles.margin} />
        </div>
    </div>
}