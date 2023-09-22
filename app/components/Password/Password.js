'use client'

import { useState } from 'react'
import styles from './password.module.css'

export default function PwdModal({onCancel, onSuccess, error}) {

    const [password, setPassword] = useState('');

    return <div className={styles.modal}>
        <div className={styles.content}>
            <div className={styles.margin} />
            <div className={styles.contents}>
                <div className={styles.header}>
                    <p>This file is password protected</p>
                </div>
                <div className={styles.field}>
                    <input value={password} onChange={(event) => setPassword(event.target.value)} type={'text'} placeholder={'Type your password'} />
                    <p className={styles.error}>{error}</p>
                </div>
                <div className={styles.buttons}>
                    <h4 onClick={onCancel}>Cancel</h4>
                    <button onClick={() => onSuccess(password)}>Open</button>
                </div>
            </div>
            <div className={styles.margin} />
        </div>
    </div>
}