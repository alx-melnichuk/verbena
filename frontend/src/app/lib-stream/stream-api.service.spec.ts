import { TestBed } from '@angular/core/testing';

import { StreamApiService } from './stream-api.service';

describe('StreamApiService', () => {
  let service: StreamApiService;

  beforeEach(() => {
    TestBed.configureTestingModule({});
    service = TestBed.inject(StreamApiService);
  });

  it('should be created', () => {
    expect(service).toBeTruthy();
  });
});
