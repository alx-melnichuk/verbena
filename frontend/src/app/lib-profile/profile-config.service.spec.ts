import { TestBed } from '@angular/core/testing';

import { ProfileConfigService } from './profile-config.service';

describe('ProfileConfigService', () => {
  let service: ProfileConfigService;

  beforeEach(() => {
    TestBed.configureTestingModule({});
    service = TestBed.inject(ProfileConfigService);
  });

  it('should be created', () => {
    expect(service).toBeTruthy();
  });
});
