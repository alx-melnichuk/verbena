import { TestBed } from '@angular/core/testing';

import { StreamScheduleService } from './stream-schedule.service';

describe('StreamScheduleService', () => {
  let service: StreamScheduleService;

  beforeEach(() => {
    TestBed.configureTestingModule({});
    service = TestBed.inject(StreamScheduleService);
  });

  it('should be created', () => {
    expect(service).toBeTruthy();
  });
});
