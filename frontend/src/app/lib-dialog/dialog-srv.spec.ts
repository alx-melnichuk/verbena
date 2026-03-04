import { TestBed } from '@angular/core/testing';

import { DialogSrv } from './dialog-srv';

describe('DialogSrv', () => {
    let service: DialogSrv;

    beforeEach(() => {
        TestBed.configureTestingModule({});
        service = TestBed.inject(DialogSrv);
    });

    it('should be created', () => {
        expect(service).toBeTruthy();
    });
});
