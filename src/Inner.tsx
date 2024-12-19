import { useState } from "react";
import { rspc } from "./App";
import reactLogo from "./assets/react.svg";
import "./App.css";

export default function Inner() {
  const [name, setName] = useState("");
  const [greetMsg, setGreetMsg] = useState("");
  const { mutate } = rspc.useMutation("greet", {
    onError(err) {
      console.error(err);
    },
    onSuccess(data) {
      console.log(data);
      setGreetMsg(data);
    },
  });
  rspc.useSubscription(["test_event"], {
    onError(err) {
      console.error(err);
    },
    onData(data) {
      console.log(data);
    },
    onStarted() {
      console.log("started 123");
    },
  });
  async function greet() {
    // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
    await mutate(name);
  }
  return (
    <main className='container'>
      <h1>Welcome to Tauri + React</h1>

      <div className='row'>
        <a href='https://vitejs.dev' target='_blank'>
          <img src='/vite.svg' className='logo vite' alt='Vite logo' />
        </a>
        <a href='https://tauri.app' target='_blank'>
          <img src='/tauri.svg' className='logo tauri' alt='Tauri logo' />
        </a>
        <a href='https://reactjs.org' target='_blank'>
          <img src={reactLogo} className='logo react' alt='React logo' />
        </a>
      </div>
      <p>Click on the Tauri, Vite, and React logos to learn more.</p>

      <form
        className='row'
        onSubmit={(e) => {
          e.preventDefault();
          greet();
        }}
      >
        <input
          id='greet-input'
          onChange={(e) => setName(e.currentTarget.value)}
          placeholder='Enter a name...'
        />
        <button type='submit'>Greet</button>
      </form>
      <p>{greetMsg}</p>
    </main>
  );
}
