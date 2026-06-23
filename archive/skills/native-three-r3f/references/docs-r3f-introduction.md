# Introduction

React-three-fiber is a React renderer for three.js.

## Summary

Build your scene declaratively with re-usable, self-contained components that react to state, are readily interactive and can participate in [React](https://react.dev/)'s ecosystem.

[![React Three Fiber banner](https://github.com/pmndrs/react-three-fiber/raw/master/docs/banner-r3f.jpg)](https://r3f.docs.pmnd.rs/getting-started/examples)

```bash
npm install three @types/three @react-three/fiber
```

Warning

Three-fiber is a React renderer, it must pair with a major version of React, just like react-dom, react-native, etc. @react-three/fiber@8 pairs with react@18, @react-three/fiber@9 pairs with react@19.

[**Does it have limitations?**](https://r3f.docs.pmnd.rs/getting-started/introduction#does-it-have-limitations?)

None. Everything that works in Threejs will work here without exception.

[**Is it slower than plain Threejs?**](https://r3f.docs.pmnd.rs/getting-started/introduction#is-it-slower-than-plain-threejs?)

No. There is no overhead. Components render outside of React. It outperforms Threejs in scale due to React's scheduling abilities.

[**Can it keep up with frequent feature updates to Threejs?**](https://r3f.docs.pmnd.rs/getting-started/introduction#can-it-keep-up-with-frequent-feature-updates-to-threejs?)

Yes. It merely expresses Threejs in JSX, `<mesh />` dynamically turns into `new THREE.Mesh()`. If a new Threejs version adds, removes or changes features, it will be available to you instantly without depending on updates to this library.

[**What does it look like?**](https://r3f.docs.pmnd.rs/getting-started/introduction#what-does-it-look-like?)

Let's make a re-usable component that has its own state, reacts to user-input and participates in the render-loop:

index.jsx

styles.css

import{createRoot}from'react-dom/client'

importReact,{useRef,useState}from'react'

import{Canvas,useFrame}from'@react-three/fiber'

import'./styles.css'

functionBox(props){

// This reference will give us direct access to the mesh

constmeshRef = useRef()

// Set up state for the hovered and active state

const\[hovered,setHover\] = useState(false)

const\[active,setActive\] = useState(false)

// Subscribe this component to the render-loop, rotate the mesh every frame

useFrame((state,delta)=>(meshRef.current.rotation.x += delta))

// Return view, these are regular three.js elements expressed in JSX

return(

<mesh

{...props}

ref={meshRef}

scale={active ? 1.5 : 1}

onClick={(event)=>setActive(!active)}

onPointerOver={(event)=>setHover(true)}

onPointerOut={(event)=>setHover(false)}>

<boxGeometryargs={\[1,1,1\]}/>

<meshStandardMaterialcolor={hovered ? 'hotpink' : 'orange'}/>

</mesh>

)

}

createRoot(document.getElementById('root')).render(

<Canvas>

<ambientLightintensity={Math.PI / 2}/>

<spotLightposition={\[10,10,10\]}angle={0.15}penumbra={1}decay={0}intensity={Math.PI}/>

<pointLightposition={\[-10, -10, -10\]}decay={0}intensity={Math.PI}/>

<Boxposition={\[-1.2,0,0\]}/>

<Boxposition={\[1.2,0,0\]}/>

</Canvas>,

Sandbox - CodeSandbox

Refresh previewOpen on CodeSandboxOpen Sandbox

Open on CodeSandboxOpen Sandbox

\[3/3\] Starting

Show TypeScript example

```bash
npm install @types/three
```

styles.css

index.tsx

```tsx
import * as THREE from 'three'
import { createRoot } from 'react-dom/client'
import React, { useRef, useState } from 'react'
import { Canvas, useFrame, ThreeElements } from '@react-three/fiber'
import './styles.css'

function Box(props: ThreeElements['mesh']) {
const meshRef = useRef<THREE.Mesh>(null!)
const [hovered, setHover] = useState(false)
const [active, setActive] = useState(false)
useFrame((state, delta) => (meshRef.current.rotation.x += delta))
return (
  <mesh
    {...props}
    ref={meshRef}
    scale={active ? 1.5 : 1}
    onClick={(event) => setActive(!active)}
    onPointerOver={(event) => setHover(true)}
    onPointerOut={(event) => setHover(false)}>
    <boxGeometry args={[1, 1, 1]} />
    <meshStandardMaterial color={hovered ? 'hotpink' : '#2f74c0'} />
  </mesh>
)
}

createRoot(document.getElementById('root')).render(
<Canvas>
  <ambientLight intensity={Math.PI / 2} />
  <spotLight position={[10, 10, 10]} angle={0.15} penumbra={1} decay={0} intensity={Math.PI} />
  <pointLight position={[-10, -10, -10]} decay={0} intensity={Math.PI} />
  <Box position={[-1.2, 0, 0]} />
  <Box position={[1.2, 0, 0]} />
</Canvas>,
)
```

[Open on CodeSandboxOpen Sandbox](https://codesandbox.io/api/v1/sandboxes/define?parameters=N4IgZglgNgpgziAXKOAXAnrOA6AxnBZEXAewDtUYKkQALVAWygBoAjEgE3WYGIAnEiVQACYLRgQA5vUQBGAAzyApAG4GAQz6SIZRAFcycGKhUBfEMxAQ4AIR2b0SMOqhHTlnRxgAPbKjjeSKCkFFSoNBAMAA4kfCIAVMLqcMIAKgASAEoAotnCYAIMwgDkqLR8MDDFADpkkTFxosK4FeqUmYIipvmFJa24qAC0HCQMAPS4UBBhNXXRsSKZMOoDzE16RktgaxswAMqobTDC3QWjfcsDs_ULTQDC6mQAbsk7RgBifOoMMGup5ZVsrAfhQUqdesUAAL9IZlCowMaQVgwPjXeaNYrYMZoTDwPAEWa1MAGAYQcjCGwkbwACiiAiicEQaQBMCBMBB_gA2sUfnBaMUALoASlEtRCaGEvNoW2EAF5hLstgAeDI5bLYACy8FoAD5qWQ9FAoABCIVi8gSzm0EhPFEwDhrIyodI2lECuUKowHI7U5yuGBmsjikSclaoCC2x3GACCpNt7vlu29lF9LiMgd2n2-MGp1LQRzWXighxFsp1wmpUq2eD0fAqFGwAkO4fI2G8wgA1PKiyXAxVULWyBXasJhEqpSPR6JsDO6SQGaZJ6OKmBZcAqzAwIuh1O4LgXDA12GI8cAPzCWTYACswiZsm3U-E5DuU1wAGs19SYLaKKXy07Y3DW1qWNY9bSFB8p3IAAFEgdEoPgAHlbT4T9vzCP9hCdF0UOpVA-D0ANINHGC4NCJC9FQNCf1QTDsNdPhU39CCdSXMd2G8ABxGBRmMPh0CSLQ4DXTlZDWMSLwFboxlYndR3HbVvTIDhNA4DUjj4CAXGaEgoFiNdrRQ-1hHPYprVQKIdDfYpbxKHgACYwAAdgAFlweRimk2T5LGKVZMDbcxVadpOmpEZcD0DlsEkYw2Q5Gx0AASQ4aligEIRiiFIVGyoLxGNqJUHmeZJvLHb5WGmCgABkpHoYR4KoOAIAwNd1LKbBoMS4QxmEeyvMnJU4BiVAaukEQYialsyBEhRxPkOapKSMhJFgNd5GwWQr26KIqEi1gvjXe9hC8fd0DW7oGsMZqzuANraA6xL-p3JUYng0a6om5qyWm4BOUGWbhH--bAYURaTvUG75Au0Irpa262nuzqnvkyl20-qaRP-7B7LWYH5EWmSBtR4R0e-mbsdx3GCdkpUxiKl44B1ZhaiFCwrFsex-KcNMYHcEAohWN91Bi7AACs4HIIJiHIShqEQEBgEnaoQDIbNlaZZW4EeDgBffQY51FmABmV5md2VjQdHV4RlbGTwfD8AITaVkAvB25SqFwaY4CtxW5OtkAYSt5WoCONAnb95WYWGUYg5AEPKDDiw2OVuFKlj-P4FQcPH2VyEMB2uAxlTmB09DrOk4jkBoUuWEWURCBkT4UuE_Lyd3GdrwngAERgN2vCDL2fe3cwPA51WucQP03FMUwgA&query=file%3D%252Findex.tsx%26utm_medium%3Dsandpack&environment=undefined "Open in CodeSandbox")

Show React Native example

For installation instructions see [react native installation instructions](https://r3f.docs.pmnd.rs/getting-started/installation#react-native).

```jsx
import React, { useRef, useState } from 'react'
import { Canvas, useFrame } from '@react-three/fiber/native'

function Box(props) {
  const meshRef = useRef(null)
  const [hovered, setHover] = useState(false)
  const [active, setActive] = useState(false)
  useFrame((state, delta) => (meshRef.current.rotation.x += delta))
  return (
    <mesh
      {...props}
      ref={meshRef}
      scale={active ? 1.5 : 1}
      onClick={(event) => setActive(!active)}
      onPointerOver={(event) => setHover(true)}
      onPointerOut={(event) => setHover(false)}>
      <boxGeometry args={[1, 1, 1]} />
      <meshStandardMaterial color={hovered ? 'hotpink' : 'orange'} />
    </mesh>
  )
}

export default function App() {
  return (
    <Canvas>
      <ambientLight intensity={Math.PI / 2} />
      <spotLight position={[10, 10, 10]} angle={0.15} penumbra={1} decay={0} intensity={Math.PI} />
      <pointLight position={[-10, -10, -10]} decay={0} intensity={Math.PI} />
      <Box position={[-1.2, 0, 0]} />
      <Box position={[1.2, 0, 0]} />
    </Canvas>
  )
}
```

[**First steps**](https://r3f.docs.pmnd.rs/getting-started/introduction#first-steps)

You need to be versed in both React and Threejs before rushing into this. If you are unsure about React consult the official [React docs](https://react.dev/learn), especially [the section about hooks](https://react.dev/reference/react). As for Threejs, make sure you at least glance over the following links:

1. Make sure you have a [basic grasp of Threejs](https://threejs.org/docs/index.html#manual/en/introduction/Creating-a-scene). Keep that site open.
2. When you know what a scene is, a camera, mesh, geometry, material, fork the [demo above](https://github.com/pmndrs/react-three-fiber#what-does-it-look-like).
3. [Look up](https://threejs.org/docs/index.html#api/en/objects/Mesh) the JSX elements that you see (mesh, ambientLight, etc), _all_ threejs exports are native to three-fiber.
4. Try changing some values, scroll through our [API](https://r3f.docs.pmnd.rs/api/canvas) to see what the various settings and hooks do.

Some helpful material:

- [Threejs-docs](https://threejs.org/docs) and [examples](https://threejs.org/examples)
- [Discover Threejs](https://discoverthreejs.com/), especially the [Tips and Tricks](https://discoverthreejs.com/tips-and-tricks) chapter for best practices
- [Bruno Simons Threejs Journey](https://threejs-journey.com/), arguably the best learning resource, now includes a full [R3F chapter](https://threejs-journey.com/lessons/what-are-react-and-react-three-fiber)

[![](https://github.com/pmndrs/react-three-fiber/raw/master/docs/banner-journey.jpg)](https://threejs-journey.com/)[**Ecosystem**](https://r3f.docs.pmnd.rs/getting-started/introduction#ecosystem)

There is a vibrant and extensive ecosystem around three-fiber, full of libraries, helpers and abstractions.

- [`@react-three/drei`](https://github.com/pmndrs/drei) – useful helpers, this is an ecosystem in itself
- [`@react-three/gltfjsx`](https://github.com/pmndrs/gltfjsx) – turns GLTFs into JSX components
- [`@react-three/postprocessing`](https://github.com/pmndrs/react-postprocessing) – post-processing effects
- [`@react-three/test-renderer`](https://github.com/pmndrs/react-three-fiber/tree/master/packages/test-renderer) – for unit tests in node
- [`@react-three/flex`](https://github.com/pmndrs/react-three-flex) – flexbox for react-three-fiber
- [`@react-three/xr`](https://github.com/pmndrs/react-xr) – VR/AR controllers and events
- [`@react-three/csg`](https://github.com/pmndrs/react-three-csg) – constructive solid geometry
- [`@react-three/rapier`](https://github.com/pmndrs/react-three-rapier) – 3D physics using Rapier
- [`@react-three/cannon`](https://github.com/pmndrs/use-cannon) – 3D physics using Cannon
- [`@react-three/p2`](https://github.com/pmndrs/use-p2) – 2D physics using P2
- [`@react-three/a11y`](https://github.com/pmndrs/react-three-a11y) – real a11y for your scene
- [`@react-three/gpu-pathtracer`](https://github.com/pmndrs/react-three-gpu-pathtracer) – realistic path tracing
- [`create-r3f-app next`](https://github.com/pmndrs/react-three-next) – nextjs starter
- [`lamina`](https://github.com/pmndrs/lamina) – layer based shader materials
- [`zustand`](https://github.com/pmndrs/zustand) – flux based state management
- [`jotai`](https://github.com/pmndrs/jotai) – atoms based state management
- [`valtio`](https://github.com/pmndrs/valtio) – proxy based state management
- [`react-spring`](https://github.com/pmndrs/react-spring) – a spring-physics-based animation library
- [`framer-motion-3d`](https://www.framer.com/docs/three-introduction/) – framer motion, a popular animation library
- [`use-gesture`](https://github.com/pmndrs/react-use-gesture) – mouse/touch gestures
- [`leva`](https://github.com/pmndrs/leva) – create GUI controls in seconds
- [`maath`](https://github.com/pmndrs/maath) – a kitchen sink for math helpers
- [`miniplex`](https://github.com/hmans/miniplex) – ECS (entity management system)
- [`composer-suite`](https://github.com/hmans/composer-suite) – composing shaders, particles, effects and game mechanics

[**How to contribute**](https://r3f.docs.pmnd.rs/getting-started/introduction#how-to-contribute)

If you like this project, please consider helping out. All contributions are welcome as well as donations to [Opencollective](https://opencollective.com/react-three-fiber), or in crypto `BTC: 36fuguTPxGCNnYZSRdgdh6Ea94brCAjMbH`, `ETH: 0x6E3f79Ea1d0dcedeb33D3fC6c34d2B1f156F2682`.

[**Backers**](https://r3f.docs.pmnd.rs/getting-started/introduction#backers)

Thank you to all our backers! 🙏

[![](https://opencollective.com/react-three-fiber/backers.svg?width=890)](https://opencollective.com/react-three-fiber#backers) [**Contributors**](https://r3f.docs.pmnd.rs/getting-started/introduction#contributors)

This project exists thanks to all the people who contribute.

[![](https://opencollective.com/react-three-fiber/contributors.svg?width=890)](https://github.com/pmndrs/react-three-fiber/graphs/contributors)

[Edit this page](https://github.com/pmndrs/react-three-fiber/edit/master/docs/getting-started/introduction.mdx)

Next

[Installation](https://r3f.docs.pmnd.rs/getting-started/installation)

- [getting started](https://r3f.docs.pmnd.rs/getting-started/introduction)









  - [Installation](https://r3f.docs.pmnd.rs/getting-started/installation)
  - [Your first scene](https://r3f.docs.pmnd.rs/getting-started/your-first-scene)
  - [Examples](https://r3f.docs.pmnd.rs/getting-started/examples)
  - [Community R3F Components](https://r3f.docs.pmnd.rs/getting-started/community-r3f-components)

- [api](https://r3f.docs.pmnd.rs/api/canvas)

- [advanced](https://r3f.docs.pmnd.rs/advanced/scaling-performance)

- [tutorials](https://r3f.docs.pmnd.rs/tutorials/v9-migration-guide)


On This Page

#### [Does it have limitations?](https://r3f.docs.pmnd.rs/getting-started/introduction\#does-it-have-limitations?)

#### [Is it slower than plain Threejs?](https://r3f.docs.pmnd.rs/getting-started/introduction\#is-it-slower-than-plain-threejs?)

#### [Can it keep up with frequent feature updates to Threejs?](https://r3f.docs.pmnd.rs/getting-started/introduction\#can-it-keep-up-with-frequent-feature-updates-to-threejs?)

#### [What does it look like?](https://r3f.docs.pmnd.rs/getting-started/introduction\#what-does-it-look-like?)

#### [First steps](https://r3f.docs.pmnd.rs/getting-started/introduction\#first-steps)

#### [Ecosystem](https://r3f.docs.pmnd.rs/getting-started/introduction\#ecosystem)

#### [How to contribute](https://r3f.docs.pmnd.rs/getting-started/introduction\#how-to-contribute)

#### [Backers](https://r3f.docs.pmnd.rs/getting-started/introduction\#backers)

#### [Contributors](https://r3f.docs.pmnd.rs/getting-started/introduction\#contributors)
