import 'reflect-metadata';

import express, { Express, Request, Response } from 'express';

import { AppDataSource } from './data-source';

import { config } from './config';
import { PlayRepository } from './repository/play';
import { CasperClient, DeployUtil } from "casper-js-sdk";

const app: Express = express();
const port = config.httpPort;

const client = new CasperClient("http://135.181.14.226:7777/rpc");

(async function () {
  await AppDataSource.initialize();

  const playsRepository = new PlayRepository(AppDataSource);

  app.get('/plays', async (req: Request, res: Response) => {
    const plays = await playsRepository.findAll();

    res.json({ data: plays });
  });

  app.listen(port, () => {
    console.log(`[server]: Server is running at http://localhost:${port}`);
  });
})();

app.get('/deploy', async (req: Request, res: Response) => {
  try {
		const deploy = DeployUtil.deployFromJson(req.body).unwrap();
		const deployHash = await client.putDeploy(deploy);
		res.send(deployHash);
	} catch (error) {
		res.status(400).send(error.message);
	}
});

process.on('SIGINT', shutdown);
process.on('SIGTERM', shutdown);

async function shutdown() {
  try {
    await AppDataSource.destroy();

    process.exit(0);
  } catch (err) {
    console.log(`received error during graceful shutdown process: ${err.message}`);
    process.exit(1);
  }
}
