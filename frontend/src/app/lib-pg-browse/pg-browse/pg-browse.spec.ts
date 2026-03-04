import { ComponentFixture, TestBed } from '@angular/core/testing';

import { PgBrowse } from './pg-browse';

describe('PgBrowse', () => {
  let component: PgBrowse;
  let fixture: ComponentFixture<PgBrowse>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [PgBrowse]
    })
    .compileComponents();

    fixture = TestBed.createComponent(PgBrowse);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
