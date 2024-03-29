import { DataSource, Repository } from 'typeorm';
import { Round } from '../entity/round.entity';

export class RoundRepository extends Repository<Round> {
  constructor(private dataSource: DataSource) {
    super(Round, dataSource.createEntityManager());
  }
}