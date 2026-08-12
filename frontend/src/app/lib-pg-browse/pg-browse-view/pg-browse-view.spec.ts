import { ComponentFixture, TestBed } from '@angular/core/testing';

import { PgBrowseView } from './pg-browse-view';

describe('PgBrowseView', () => {
  let component: PgBrowseView;
  let fixture: ComponentFixture<PgBrowseView>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [PgBrowseView]
    })
    .compileComponents();

    fixture = TestBed.createComponent(PgBrowseView);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
