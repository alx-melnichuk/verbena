import { TestBed } from '@angular/core/testing';

import { StreamCalendarService } from './stream-calendar.service';

describe('StreamCalendarService', () => {
  let service: StreamCalendarService;

  beforeEach(() => {
    TestBed.configureTestingModule({});
    service = TestBed.inject(StreamCalendarService);
  });

  it('should be created', () => {
    expect(service).toBeTruthy();
  });
});
