import { TestBed } from '@angular/core/testing';

import { StreamListService } from './stream-list.service';

describe('StreamListService', () => {
  let service: StreamListService;

  beforeEach(() => {
    TestBed.configureTestingModule({});
    service = TestBed.inject(StreamListService);
  });

  it('should be created', () => {
    expect(service).toBeTruthy();
  });
});
