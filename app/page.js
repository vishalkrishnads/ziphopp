'use client'

import styles from './page.module.css'
import { invoke } from '@tauri-apps/api/tauri'
import { useState, useRef } from 'react'

export default function Home() {

  const [password, setPassword] = useState('');
  const [contents, setContents] = useState([])
  const [history, setHistory] = useState([])
  const [error, setError] = useState('No errors');
  const path = useRef('')

  // appWindow.listen('password', (event) => {
  //   setStatus('You need a password for this')
  // })

  const openWithPassword = () => {
    invoke('open_file', {
      path: path.current,
      password: password
    })
    .then((result) => {
      setContents(result.contents);
      refresh();
    })
    .catch((error) => setError(JSON.stringify(error.message)))
  }

  const refresh = () => {
    invoke('refresh')
    .then((result) => setHistory(result.history))
    .catch((error) => setError(error.message))
  }

  return (
    <main className={styles.main}>
      <div className={styles.left}>
        <div className={styles.top}>
          <input value={password} onChange={e => setPassword(e.target.value)} />
          <button style={{ marginLeft: '10px' }} onClick={openWithPassword}>Go</button>
        </div>
        <div className={styles.middle}>
          <button onClick={() => {
            invoke('open_file').then((result) => {
              setContents(result.contents);
              refresh();
            })
            .catch((error) => {
              setError(error.message)
              if(error.password_required) path.current = error.path
            })
          }}>Open file</button>
          <p>{error}</p>
        </div>
        <div className={styles.bottom}>
          {history.map((item, index) => {
            return <div onClick={() => {
              invoke('open_file', {
                path: item.path,
              }).then((result) => {
                setContents(result.contents);
                refresh();
              })
              .catch((error) => {
                setError(error.message)
                if(error.password_required) path.current = error.path
              })
            }}>
              <p>{item.name}</p>
              <p>{item.path}</p>
            </div>
          })}
          <button onClick={refresh} >Refresh</button>
        </div>
      </div>
      <div className={styles.right}>
        {contents.map((item, index) => {
          return <p>{item}</p>
        })}
      </div>
    </main>
  )
}
