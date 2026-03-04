import { ComponentFixture, TestBed } from '@angular/core/testing';

import { PgSignup } from './pg-signup';

describe('PgSignup', () => {
  let component: PgSignup;
  let fixture: ComponentFixture<PgSignup>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [PgSignup]
    })
    .compileComponents();

    fixture = TestBed.createComponent(PgSignup);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
