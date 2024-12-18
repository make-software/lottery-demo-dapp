import 'reflect-metadata';

import { AppDataSource } from '../data-source';
import fs from 'fs/promises';
import path from 'path';
import { Play } from '../entity/play.entity';

(async () => {
  try {
    await AppDataSource.initialize();

    const data = await fs.readFile(path.join(__dirname, 'plays.json'), 'utf8');

    const plays = JSON.parse(data) as Play[];

    const playsRepository = AppDataSource.getRepository(Play);

    await playsRepository.save(plays);

    AppDataSource.destroy();

    process.exit(0);
  } catch (err) {
    console.log(err);
    process.exit(1);
  }
})();
