import { ComponentFixture, TestBed } from '@angular/core/testing';

import { PgProfile } from './pg-profile';

describe('PgProfile', () => {
  let component: PgProfile;
  let fixture: ComponentFixture<PgProfile>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [PgProfile]
    })
    .compileComponents();

    fixture = TestBed.createComponent(PgProfile);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
