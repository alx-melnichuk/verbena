import { TestBed } from '@angular/core/testing';

import { StreamConfigService } from './stream-config.service';

describe('StreamConfigService', () => {
  let service: StreamConfigService;

  beforeEach(() => {
    TestBed.configureTestingModule({});
    service = TestBed.inject(StreamConfigService);
  });

  it('should be created', () => {
    expect(service).toBeTruthy();
  });
});
