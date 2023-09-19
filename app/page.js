'use client'

import Image from 'next/image'
import Header from './components/Header/Header'
import Recent from './components/Recent/Recent'
import PwdModal from './components/Password/Password'
import styles from './page.module.css'
import zip from './assets/zipicon.png'
import { useEffect, useRef, useState } from 'react'
import { invoke } from '@tauri-apps/api'

export default function Home() {

  const [modal, setModal] = useState(false);
  const [current, setCurrent] = useState({});
  const [contents, setContents] = useState([])
  const [recents, setRecents] = useState([])
  const pathRef = useRef('');

  const openFile = (path = '', password = '') => {
    // clear the current file
    setCurrent({})

    // setup the payload
    let payload = {}
    if(password.length > 0) payload.password = password
    if(path.length > 0) payload.path = path

    invoke('open_file', payload)
    .then((result) => {
      setCurrent(result);
      setContents(result.contents)
      refresh()
    })
    .catch((error) => {
      if(error.password_required) {
        setModal(true);
        pathRef.current = error.path
      }
    })
  }

  const refresh = () => {
    invoke('refresh')
    .then((result) => setRecents(result.history))
    .catch((error) => {})
  }

  useEffect(refresh, [])

  return (
    <main className={styles.main}>
      {modal ? <PwdModal onCancel={() => setModal(false)} onSuccess={(password) => {setModal(false); openFile(pathRef.current, password)}} /> : null}
      <div className={styles.app}>
        <div className={styles.left}>
          <div className={styles.contents}>
            <Header />
            <div className={styles.filepane}>
              {Object.getOwnPropertyNames(current).length == 0 ?
               <button onClick={openFile}>Open file</button> :
               <div className={styles.file}>
                <div className={styles.margin} />
                <div className={styles.icon}>
                  <Image
                    src={zip}
                    alt={''}
                    unoptimized
                  />
                </div>
                <div className={styles.info}>
                  <h4>{current.meta.name}</h4>
                  <h6>{current.path}</h6>
                  <p>{current.meta.compressed}, uncompresses to {current.meta.size}</p>
                  <div onClick={openFile}><p>Open another</p></div>
                </div>
                <div className={styles.margin} />
              </div>}
            </div>
            <div className={styles.recentpane}>
              <div className={styles.margin} />
              <div className={styles.pane}>
                <div className={styles.header}>
                  <h3>{recents.length > 0 ? 'You previously opened...' : 'Recent files'}</h3>
                </div>
                {recents.length > 0 ?
                <div className={styles.recents}>
                  {recents.map((item, index) => {
                    return <Recent key={index} name={item.name} path={item.path} onClick={() => openFile(item.path)} />
                  })}
                </div> 
                : <div className={styles.recents} style={{ justifyContent: 'center' }} >
                  <p>Files you open will appear here</p>
                </div>}
              </div>
              <div className={styles.margin} />
            </div>
          </div>
        </div>
        <div className={styles.right}>
          {contents.map((item, index) => {
            return <p key={index}>{item}</p>
          })}
        </div>
      </div>
    </main>
  )
}
