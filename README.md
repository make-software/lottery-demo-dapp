# Casper Lottery
This repository contains the front-end implementation of a decentralized application (DApp) demonstrating a simple lottery system using CSPR.suite series of products.

Built with CSPR.click, Odra, and the CSPR.cloud, this DApp showcases how Casper blockchain technology can be utilized for transparent and fair random selection processes.

Detailed spec:  [here](./docs/specs.md)

### Setup 
#### CSPR.click
Create a new project using CSPR.click react template:
```
npx create-react-app my-new-casper-app --template @make-software/csprclick-react
```

Go to the newly created project directory, install dependencies and run the app. 

In this case it will be "my-new-casper-app"
```
cd my-new-casper-app
npm install
npm start
```
This command will start the development server. You can view the app by navigating to http://localhost:3000 in your web browser.

More details [here](https://docs.cspr.click/) 

[CRA link](https://www.npmjs.com/package/@make-software/cra-template-csprclick-react)

### Build
To build the project for production, use:
```
npm run build
```
This command will create a build folder with optimized production-ready files.

### Usage
To initialize and use the CSPR.click feature in the frontend application, follow these steps:
1. ####Import the hook:
In the React component file where you want to use the CSPR.click feature, import the hook from the library:
```
import { useClickRef } from '@make-software/csprclick-ui';
```
2. ####Initialize the hook:
```
const clickRef = useClickRef();
```
3. ####Use methods to handle sign in/sign out events
```
clickRef.on('csprclick:signed_in', async (event) => await doSomethingWithAccount(event.account));
```
4. ####Add the CSPR.click header
To add the CSPR.click header to your application import the ClickUI component. 

```
import { ClickUI } from '@make-software/csprclick-ui';
```
Customize the behavior of the hook by passing options as parameters. For example, you can use advanced functions such as adding a list of languages, a dark theme, and a list of currencies by passing parameters to props.
```
<ClickUI
	topBarSettings={{
		languageSettings: languageSettings
	}}
/>
```
