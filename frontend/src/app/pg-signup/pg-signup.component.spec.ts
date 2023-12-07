import { ComponentFixture, TestBed } from '@angular/core/testing';

import { PgSignupComponent } from './pg-signup.component';

describe('PgSignupComponent', () => {
  let component: PgSignupComponent;
  let fixture: ComponentFixture<PgSignupComponent>;

  beforeEach(() => {
    TestBed.configureTestingModule({
      imports: [PgSignupComponent]
    });
    fixture = TestBed.createComponent(PgSignupComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
