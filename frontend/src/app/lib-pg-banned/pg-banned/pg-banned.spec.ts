import { ComponentFixture, TestBed } from '@angular/core/testing';

import { PgBanned } from './pg-banned';

describe('PgBanned', () => {
  let component: PgBanned;
  let fixture: ComponentFixture<PgBanned>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [PgBanned]
    })
    .compileComponents();

    fixture = TestBed.createComponent(PgBanned);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
