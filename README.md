# Equilibrium

<div align="center">
  <a href="https://github.com/vkabadzhova/equilibrium/releases">
    <img alt="GitHub release (latest by date including pre-releases)" src="https://img.shields.io/github/v/release/vkabadzhova/equilibrium?include_prereleases&label=latest"></a>
  <a href="https://circleci.com/gh/badges/shields/tree/master">
        <img src="https://img.shields.io/circleci/project/github/badges/shields/master" alt="build status"></a>
    <a href="https://github.com/vkabadzhova/equilibrium/actions/workflows/rust.yml">
  <a href="https://github.com/vkabadzhova/equilibrium/blob/main/LICENSE"><img src="https://img.shields.io/github/license/vkabadzhova/equilibrium.svg"></a>
</div>

  
> A CPU-processed and grid-based fluid simulator built for my diploma thesis.
  
Hello and welcome to Equilibrium - a light version of a fluid simulator (written fully in Rust) and also my high school diploma thesis!

It is written using the [egui](https://github.com/emilk/egui) library. The fluid simulator is based on the Jos Stam's paper from 2003 called ["Real-Time Fluid Dynamics for Games"](http://graphics.cs.cmu.edu/nsp/course/15-464/Fall09/papers/StamFluidforGames.pdf). However, the current simulator is not real time and is CPU-based.

![Fluid simulation video](https://i.imgur.com/G4GyMIX.gif)  
  
Sections:
* [Dependencies](#dependencies)
* [Demo](#demo)
* [State / features](#features)
* [Official Documentation (in Bulgarian only)](#documentation)
* [Technical Documentation](#technical-documentation)
* [Vision](#vision)
  
## Dependencies
All the dependencies are described in the `Cargo.toml` file. When the application is run by `cargo r` they will be automatically fetched. 

## Demo
You can find a demonstration of the project on GitHub pages or on [vkabadzhova.xyz](vkabadzhova.xyz). It is compiled to WASM.
  
## Features
- Scene (fluid simulation)
  - only 2D simulation is supported;
  - The colors of the fluid and the world around it can be altered;
  - Obstacles can be set in the scene (only rectangle shapes are supported)
- Application
  - Dark/Light theme
  - Navigation through the simulation (Next, previous frame, scroll through the whole application)
  - Configurations through the GUI:
    - simulation settings: number of frames, speed of the simulation,
    - fluid settings: diffusion, viscousity and colour,
    - obstacles settings: add/remove/change shape of the obstacles
    - viewport settings: zoom in/out
  
## Documentation
> Note: the official documentation of the project is available only in Bulgarian. The translation is unofficial.

The official documentation is written in LaTex and can be found [here](https://drive.google.com/file/d/1OVoyD_1qSvfdFf7lj_Z880GoMIcrcszu/view?usp=share_link), the home-brewed translation is available [here](https://drive.google.com/file/d/15Y4aMHqkBHwbQ5CsVRgF_BfI0RH30nvR/view?usp=share_link).

### What can you find in the documentation?
- Chapter 1: Available research on the topic of simulating fluid dynamics;
- Chapter 2: Selecting the technology stack - the programming language and algorithms;
- Chapter 3: Implementation details;
- Chapter 4: User guide.

## Technical Documentation
Typically, using the Rust's packet manager `Cargo` a technical documentation can be generated via `cargo d` or `cargo d --open` if you want to open it directly. 

## Vision
The project as it is now is not production ready. It is designed on the grounds of educational interest in the field of computar graphics and the Rust programming language. In case you wish to contribute, please, feel free to do so by first reading the `CONTRIBUTING.md` file.
