import { TestBed } from '@angular/core/testing';

import { AlertSrv } from './alert-srv';

describe('AlertSrv', () => {
  let service: AlertSrv;

  beforeEach(() => {
    TestBed.configureTestingModule({});
    service = TestBed.inject(AlertSrv);
  });

  it('should be created', () => {
    expect(service).toBeTruthy();
  });
});
